use chrono::{DateTime, Duration, Utc};
use jiaoyi_core::{allowlist::Allowlist, secret::Secret};
use jiaoyi_ctrader::testkit::{m30_bar, FakeBroker};

#[tokio::test]
async fn research_fetch_refuses_truncation_and_writes_no_order() {
    let at = "2026-11-02T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
    let mut broker = FakeBroker {
        account_id: 42,
        max_bars_per_response: 1,
        ..Default::default()
    };
    broker.m30_bars.insert(
        7,
        vec![
            m30_bar(at, 100, 101, 99, 100, 1),
            m30_bar(at + Duration::minutes(30), 100, 101, 99, 100, 1),
        ],
    );
    let (conn, log) = broker.spawn();
    conn.application_auth("app", &Secret::new("secret"))
        .await
        .unwrap();
    let account = Allowlist::parse(Some("42")).unwrap().permit(42).unwrap();
    let s = conn
        .account_auth(&account, &Secret::new("token"))
        .await
        .unwrap();
    let err = s
        .research_m30_page(7, at, at + Duration::hours(1))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("truncated"));
    let log = log.lock().unwrap();
    assert!(log.orders.is_empty() && log.amends.is_empty() && log.closes.is_empty());
}
