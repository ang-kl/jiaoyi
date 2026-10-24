//! The client against the scripted in-memory proxy: real framing, real
//! protobuf messages, no broker credentials.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use chrono::{NaiveDate, TimeZone, Utc};
use jiaoyi_core::allowlist::Allowlist;
use jiaoyi_core::secret::Secret;
use jiaoyi_ctrader::messages::{ProtoOaLightSymbol, ProtoOaPayloadType};
use jiaoyi_ctrader::payload::SENDABLE;
use jiaoyi_ctrader::testkit::{d1_bar, FakeBroker};
use jiaoyi_ctrader::{closed_daily_bars, decode_trendbar, ClientError};

const ACCOUNT: i64 = 49236051;

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn fake() -> FakeBroker {
    FakeBroker {
        account_id: ACCOUNT,
        light_symbols: vec![ProtoOaLightSymbol {
            symbol_id: 1,
            symbol_name: Some("EURUSD".into()),
            enabled: Some(true),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[tokio::test]
async fn authorises_and_lists_symbols() {
    let (conn, log) = fake().spawn();
    conn.application_auth("id", &Secret::new("secret"))
        .await
        .unwrap();
    let list = Allowlist::parse(Some("49236051")).unwrap();
    let account = list.permit(ACCOUNT).unwrap();
    let session = conn
        .account_auth(&account, &Secret::new("tok"))
        .await
        .unwrap();
    let symbols = session.symbols_list().await.unwrap();
    assert_eq!(symbols.symbol[0].symbol_name.as_deref(), Some("EURUSD"));
    let seen = log.lock().unwrap().payload_types.clone();
    assert_eq!(
        seen,
        vec![
            ProtoOaPayloadType::ProtoOaApplicationAuthReq as u32,
            ProtoOaPayloadType::ProtoOaAccountAuthReq as u32,
            ProtoOaPayloadType::ProtoOaSymbolsListReq as u32,
        ]
    );
    // Every frame the fake received was on the allowlist.
    assert!(seen
        .iter()
        .all(|t| SENDABLE.iter().any(|(s, _)| *s as u32 == *t)));
}

#[tokio::test]
async fn broker_errors_come_back_as_errors_with_their_code() {
    let mut f = fake();
    f.expired_tokens = HashSet::from(["old".to_string()]);
    let (conn, _) = f.spawn();
    conn.application_auth("id", &Secret::new("s"))
        .await
        .unwrap();
    let list = Allowlist::parse(Some("49236051,11111111")).unwrap();
    let err = conn
        .account_auth(&list.permit(ACCOUNT).unwrap(), &Secret::new("old"))
        .await
        .err()
        .unwrap();
    assert!(err.is_token_rejected(), "{err}");
    // An allowlisted account the broker does not know.
    let err = conn
        .account_auth(&list.permit(11111111).unwrap(), &Secret::new("tok"))
        .await
        .err()
        .unwrap();
    assert!(
        matches!(&err, ClientError::Broker { code, .. } if code == "CH_CTID_TRADER_ACCOUNT_NOT_FOUND"),
        "{err}"
    );
}

#[tokio::test]
async fn refresh_returns_the_new_pair() {
    let mut f = fake();
    f.refresh_reply = Some(("new-access".into(), "new-refresh".into(), 2_628_000));
    let (conn, _) = f.spawn();
    conn.application_auth("id", &Secret::new("s"))
        .await
        .unwrap();
    let t = conn
        .refresh_token(&Secret::new("old-refresh"))
        .await
        .unwrap();
    assert_eq!(t.access_token.expose(), "new-access");
    assert_eq!(t.refresh_token.expose(), "new-refresh");
    assert_eq!(t.expires_in_secs, 2_628_000);
    // Debug output of the refreshed token never shows the secrets.
    let shown = format!("{t:?}");
    assert!(
        !shown.contains("new-access") && !shown.contains("new-refresh"),
        "{shown}"
    );
}

#[tokio::test]
async fn daily_history_walks_back_and_splits_on_has_more() {
    // 900 daily bars from 2022-01-01; the fake answers at most 100 per request
    // with hasMore, so the client must split windows to get all of them.
    let start = day(2022, 1, 1);
    let bars: Vec<_> = (0..900)
        .map(|i| {
            let d = start + chrono::Duration::days(i);
            d1_bar(d, 100_000 + i, 100_100 + i, 99_900 + i, 100_050 + i, 10)
        })
        .collect();
    let mut f = fake();
    f.bars = HashMap::from([(1, bars)]);
    f.max_bars_per_response = 100;
    let (conn, log) = f.spawn();
    conn.application_auth("id", &Secret::new("s"))
        .await
        .unwrap();
    let account = Allowlist::parse(Some("49236051"))
        .unwrap()
        .permit(ACCOUNT)
        .unwrap();
    let session = conn
        .account_auth(&account, &Secret::new("t"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 20, 12, 0, 0).unwrap();
    let raw = session.daily_history(1, now).await.unwrap();
    assert_eq!(raw.len(), 900);
    let decoded: Vec<_> = raw.iter().map(|b| decode_trendbar(b).unwrap()).collect();
    assert_eq!(decoded[0].date, start);
    assert_eq!(decoded[899].date, start + chrono::Duration::days(899));
    // It stopped after the empty windows before 2022, not at 1970.
    let reqs = log.lock().unwrap().trendbar_requests.len();
    assert!(reqs < 60, "{reqs} requests");
}

#[tokio::test]
async fn forming_bar_from_the_broker_is_dropped() {
    let mut f = fake();
    f.bars = HashMap::from([(
        1,
        vec![
            d1_bar(day(2024, 3, 4), 100, 110, 90, 105, 1),
            d1_bar(day(2024, 3, 5), 105, 120, 100, 118, 1), // today's, still forming
        ],
    )]);
    let (conn, _) = f.spawn();
    conn.application_auth("id", &Secret::new("s"))
        .await
        .unwrap();
    let account = Allowlist::parse(Some("49236051"))
        .unwrap()
        .permit(ACCOUNT)
        .unwrap();
    let session = conn
        .account_auth(&account, &Secret::new("t"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2024, 3, 5, 13, 0, 0).unwrap();
    let raw = session.daily_history(1, now).await.unwrap();
    let decoded: Vec<_> = raw.iter().map(|b| decode_trendbar(b).unwrap()).collect();
    let (closed, forming) = closed_daily_bars(decoded, now);
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].date, day(2024, 3, 4));
    assert_eq!(forming[0].date, day(2024, 3, 5));
}

#[tokio::test]
async fn spread_sampling_carries_sides_forward_and_takes_the_median() {
    let mut f = fake();
    // ask − bid: (none, bid only) → 20 → 30 (ask moved) → 10 (bid moved)
    f.quotes = HashMap::from([(
        1,
        vec![
            (Some(108_500), None),
            (None, Some(108_520)),
            (None, Some(108_530)),
            (Some(108_520), None),
        ],
    )]);
    let (conn, _) = f.spawn();
    conn.application_auth("id", &Secret::new("s"))
        .await
        .unwrap();
    let account = Allowlist::parse(Some("49236051"))
        .unwrap()
        .permit(ACCOUNT)
        .unwrap();
    let session = conn
        .account_auth(&account, &Secret::new("t"))
        .await
        .unwrap();
    let samples = session
        .sample_spreads(&[1], Duration::from_millis(300))
        .await
        .unwrap();
    assert_eq!(samples[&1].spreads, vec![20, 30, 10]);
    assert_eq!(
        samples[&1].median(),
        Some(rust_decimal::Decimal::new(20, 5))
    );
}
