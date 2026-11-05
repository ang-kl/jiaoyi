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

#[tokio::test]
async fn bounded_tick_fetch_splits_truncated_pages_and_preserves_duplicates_without_orders() {
    use jiaoyi::broker_backtest::{bounded_ticks, TickBudget};
    use jiaoyi_ctrader::tickdata::QuoteSide;
    let mut broker = FakeBroker {
        account_id: 42,
        max_ticks_per_response: 2,
        ..Default::default()
    };
    broker.tick_history.insert(
        (7, 1),
        vec![
            (1000, 100000),
            (1000, 100000),
            (2000, 100001),
            (3000, 100002),
        ],
    );
    // More than a page within one millisecond must fail instead of dropping ticks.
    broker
        .tick_history
        .insert((7, 2), vec![(1000, 100000), (1000, 100001), (1000, 100002)]);
    let (conn, log) = broker.spawn();
    conn.application_auth("app", &Secret::new("secret"))
        .await
        .unwrap();
    let account = Allowlist::parse(Some("42")).unwrap().permit(42).unwrap();
    let s = conn
        .account_auth(&account, &Secret::new("token"))
        .await
        .unwrap();
    let mut budget = TickBudget::default();
    let ticks = bounded_ticks(&s, 7, QuoteSide::Bid, 1000, 3000, &mut budget)
        .await
        .unwrap();
    assert_eq!(ticks.len(), 4);
    assert_eq!(ticks[0].ts_ms, 1000);
    assert_eq!(ticks[1].ts_ms, 1000);
    assert_eq!(ticks[3].ts_ms, 3000);
    assert!(budget.pages > 1);
    assert_eq!(
        bounded_ticks(&s, 7, QuoteSide::Ask, 1000, 1000, &mut budget)
            .await
            .unwrap_err(),
        "tick_truncation_not_resolvable_no_result"
    );
    budget.pages = jiaoyi_core::research::s3_ticks::MAX_PAGES;
    assert_eq!(
        bounded_ticks(&s, 7, QuoteSide::Bid, 1000, 3000, &mut budget)
            .await
            .unwrap_err(),
        "tick_page_budget_exceeded_no_result"
    );
    let log = log.lock().unwrap();
    assert!(log.orders.is_empty() && log.amends.is_empty() && log.closes.is_empty());
}
