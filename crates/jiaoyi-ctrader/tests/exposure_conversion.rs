//! №18,392: manual JPY exposure must be valued before a USD candidate.
use jiaoyi_core::{allowlist::Allowlist, secret::Secret};
use jiaoyi_ctrader::{
    margin, messages as pb,
    testkit::FakeBroker,
    trading::{Command, Side, VolumeLimits},
};
use std::collections::HashMap;
use std::time::Duration;

const ACCOUNT: i64 = 49236051;
fn symbol(id: i64, base: i64, quote: i64) -> pb::ProtoOaLightSymbol {
    pb::ProtoOaLightSymbol {
        symbol_id: id,
        base_asset_id: Some(base),
        quote_asset_id: Some(quote),
        ..Default::default()
    }
}

#[tokio::test]
async fn foreign_manual_position_warms_broker_chain_once_and_margin_check_accepts_usd_candidate() {
    let trader = pb::ProtoOaTrader {
        ctid_trader_account_id: ACCOUNT,
        balance: 100_000,
        deposit_asset_id: 1,
        money_digits: Some(2),
        ..Default::default()
    };
    let fake = FakeBroker {
        account_id: ACCOUNT,
        trader: Some(trader.clone()),
        light_symbols: vec![symbol(10, 10, 2), symbol(20, 20, 1), symbol(30, 1, 2)],
        quotes: HashMap::from([(30, vec![(Some(15_000_000), Some(15_100_000))])]),
        ..Default::default()
    };
    fake.positions.lock().unwrap().push(pb::ProtoOaPosition {
        position_id: 99,
        position_status: 1,
        price: Some(40_000.0),
        stop_loss: Some(39_850.0),
        used_margin: Some(100),
        money_digits: Some(2),
        trade_data: pb::ProtoOaTradeData {
            symbol_id: 10,
            volume: 100,
            trade_side: 1,
            ..Default::default()
        },
        ..Default::default()
    });
    let (conn, log) = fake.spawn();
    conn.application_auth("id", &Secret::new("secret"))
        .await
        .unwrap();
    let allow = Allowlist::parse(Some("49236051")).unwrap();
    let session = conn
        .account_auth(&allow.permit(ACCOUNT).unwrap(), &Secret::new("token"))
        .await
        .unwrap();
    let book = session.margin_book().await.unwrap();
    let mut spots = conn.spot_events();
    margin::warm_account_conversions(&session, &trader, &book)
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(1), spots.recv())
        .await
        .unwrap()
        .unwrap();
    margin::warm_account_conversions(&session, &trader, &book)
        .await
        .unwrap();
    let status = conn.conversion_status(ACCOUNT);
    assert_eq!(status["paths"][0]["state"], "fresh");
    assert_eq!(status["paths"][0]["subscribed"], true);
    assert_eq!(status["paths"][0]["legs"][0]["inverse"], true);
    assert_eq!(
        conn.conversion_status(ACCOUNT + 1)["paths"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let command = Command::Open {
        symbol_id: 20,
        side: Side::Buy,
        volume: 100,
        relative_stop: 100_000,
        relative_take_profit: None,
        limits: VolumeLimits {
            min: 1,
            step: 1,
            max: Some(100_000),
            digits: 2,
        },
        quote_to_deposit: Some(1.0),
        label: "jiaoyi-s3".into(),
    };
    let receipt = margin::check(&session, &trader, &book, &command, 200.0)
        .await
        .unwrap();
    let proof = receipt.fx_guard.unwrap();
    assert_eq!(proof.foreign_exposures.len(), 1);
    assert_eq!(proof.foreign_exposures[0].stop_cash_quote, -150.0);
    let q = conn.quote_snapshot(ACCOUNT, 30).unwrap();
    assert!(
        (margin::foreign_reserve(&proof.foreign_exposures, chrono::Utc::now(), |_| Some(&q))
            .unwrap()
            - 1.0)
            .abs()
            < 1e-9
    );
    let log = log.lock().unwrap();
    assert_eq!(
        log.payload_types
            .iter()
            .filter(|&&p| p == pb::ProtoOaPayloadType::ProtoOaSymbolsForConversionReq as u32)
            .count(),
        1
    );
    assert_eq!(
        log.payload_types
            .iter()
            .filter(|&&p| p == pb::ProtoOaPayloadType::ProtoOaSubscribeSpotsReq as u32)
            .count(),
        1
    );
    assert!(log.orders.is_empty());
}
