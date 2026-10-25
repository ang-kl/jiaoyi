//! The payload allowlist: the one place that names the payload types this
//! program may send to cTrader.
//!
//! Two layers stop anything else:
//! 1. **Type level.** [`Request`] is sealed and implemented only for the
//!    request messages listed in [`SENDABLE`]. `Connection::request` takes
//!    `R: Request`, so a `ProtoOaNewOrderReq` (which is not even exported from
//!    this crate) cannot be passed to it. A compile-time assertion in the tests
//!    pins that the trading requests do not implement [`Request`].
//! 2. **Runtime.** The frame writer calls [`check_sendable`] on every outgoing
//!    payload type, so even a frame built by hand is refused unless listed.
//!
//! Only read requests are listed here. The three trading requests are on a
//! separate list, [`TRADING`], which only the trading gate
//! (`crate::trading::send`) writes through: it checks [`check_tradable`], never
//! [`check_sendable`], and nothing else in the program can send them (owner,
//! 25-10-2026: the gated order path).

use crate::pb::{self, ProtoOaPayloadType as Oa, ProtoPayloadType as Common};

/// Every payload type this program may send, with the reason it is needed.
pub const SENDABLE: &[(i32, &str)] = &[
    (Common::HeartbeatEvent as i32, "keep the connection open"),
    (
        Oa::ProtoOaApplicationAuthReq as i32,
        "authorise the application",
    ),
    (
        Oa::ProtoOaAccountAuthReq as i32,
        "authorise a permitted account",
    ),
    (
        Oa::ProtoOaRefreshTokenReq as i32,
        "refresh the OAuth access token",
    ),
    (
        Oa::ProtoOaSymbolsListReq as i32,
        "list the account's symbols",
    ),
    (
        Oa::ProtoOaSymbolByIdReq as i32,
        "read symbol details and costs",
    ),
    (Oa::ProtoOaGetTrendbarsReq as i32, "read daily bars"),
    (Oa::ProtoOaSubscribeSpotsReq as i32, "sample spreads"),
    (
        Oa::ProtoOaUnsubscribeSpotsReq as i32,
        "stop sampling spreads",
    ),
    (
        Oa::ProtoOaGetTickdataReq as i32,
        "read historical bid and ask ticks",
    ),
    (
        Oa::ProtoOaDealListReq as i32,
        "read the account's deal history",
    ),
    (
        Oa::ProtoOaTraderReq as i32,
        "read the account's registration date and deposit asset",
    ),
    (
        Oa::ProtoOaGetAccountsByAccessTokenReq as i32,
        "read which accounts the access token's grant covers",
    ),
    (
        Oa::ProtoOaAssetListReq as i32,
        "read the account's asset names (quote and deposit currencies)",
    ),
    (
        Oa::ProtoOaReconcileReq as i32,
        "read the account's open positions and pending orders",
    ),
    (
        Oa::ProtoOaGetPositionUnrealizedPnlReq as i32,
        "read the open positions' unrealized P&L (fresh equity)",
    ),
];

/// The trading requests, sent only by the trading gate (`crate::trading`),
/// never by `Connection::request`.
pub const TRADING: &[(i32, &str)] = &[
    (
        Oa::ProtoOaNewOrderReq as i32,
        "open a position (MARKET, with a relative stop)",
    ),
    (
        Oa::ProtoOaAmendPositionSltpReq as i32,
        "move an owned position's stop in its favour",
    ),
    (
        Oa::ProtoOaClosePositionReq as i32,
        "close a position this program opened",
    ),
];

/// The trading gate's frame check: only a payload type on [`TRADING`].
pub fn check_tradable(payload_type: u32) -> Result<(), NotSendable> {
    if TRADING.iter().any(|(t, _)| *t as u32 == payload_type) {
        Ok(())
    } else {
        Err(NotSendable {
            payload_type,
            name: payload_name(payload_type),
        })
    }
}

/// A payload type that is not on [`SENDABLE`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("payload type {payload_type} ({name}) is not on the sendable allowlist; refused")]
pub struct NotSendable {
    pub payload_type: u32,
    pub name: &'static str,
}

/// The runtime gate. Called by the frame writer for every outgoing frame.
pub fn check_sendable(payload_type: u32) -> Result<(), NotSendable> {
    if SENDABLE.iter().any(|(t, _)| *t as u32 == payload_type) {
        Ok(())
    } else {
        Err(NotSendable {
            payload_type,
            name: payload_name(payload_type),
        })
    }
}

/// The protocol name of a payload type, for messages and records.
pub fn payload_name(payload_type: u32) -> &'static str {
    let Ok(t) = i32::try_from(payload_type) else {
        return "UNKNOWN";
    };
    if let Ok(oa) = Oa::try_from(t) {
        return oa.as_str_name();
    }
    if let Ok(c) = Common::try_from(t) {
        return c.as_str_name();
    }
    "UNKNOWN"
}

mod sealed {
    pub trait Sealed {}
}

/// A response message and its payload type.
pub trait Response: prost::Message + Default {
    const PAYLOAD_TYPE: u32;
}

/// A request this program may send. Sealed: implemented only below, only for
/// types on [`SENDABLE`].
pub trait Request: prost::Message + sealed::Sealed {
    const PAYLOAD_TYPE: u32;
    type Response: Response;
}

macro_rules! request {
    ($req:ty => $req_pt:expr, $res:ty => $res_pt:expr) => {
        impl sealed::Sealed for $req {}
        impl Request for $req {
            const PAYLOAD_TYPE: u32 = $req_pt as u32;
            type Response = $res;
        }
        impl Response for $res {
            const PAYLOAD_TYPE: u32 = $res_pt as u32;
        }
    };
}

request!(pb::ProtoOaApplicationAuthReq => Oa::ProtoOaApplicationAuthReq,
         pb::ProtoOaApplicationAuthRes => Oa::ProtoOaApplicationAuthRes);
request!(pb::ProtoOaAccountAuthReq => Oa::ProtoOaAccountAuthReq,
         pb::ProtoOaAccountAuthRes => Oa::ProtoOaAccountAuthRes);
request!(pb::ProtoOaRefreshTokenReq => Oa::ProtoOaRefreshTokenReq,
         pb::ProtoOaRefreshTokenRes => Oa::ProtoOaRefreshTokenRes);
request!(pb::ProtoOaSymbolsListReq => Oa::ProtoOaSymbolsListReq,
         pb::ProtoOaSymbolsListRes => Oa::ProtoOaSymbolsListRes);
request!(pb::ProtoOaSymbolByIdReq => Oa::ProtoOaSymbolByIdReq,
         pb::ProtoOaSymbolByIdRes => Oa::ProtoOaSymbolByIdRes);
request!(pb::ProtoOaGetTrendbarsReq => Oa::ProtoOaGetTrendbarsReq,
         pb::ProtoOaGetTrendbarsRes => Oa::ProtoOaGetTrendbarsRes);
request!(pb::ProtoOaSubscribeSpotsReq => Oa::ProtoOaSubscribeSpotsReq,
         pb::ProtoOaSubscribeSpotsRes => Oa::ProtoOaSubscribeSpotsRes);
request!(pb::ProtoOaUnsubscribeSpotsReq => Oa::ProtoOaUnsubscribeSpotsReq,
         pb::ProtoOaUnsubscribeSpotsRes => Oa::ProtoOaUnsubscribeSpotsRes);
request!(pb::ProtoOaGetTickDataReq => Oa::ProtoOaGetTickdataReq,
         pb::ProtoOaGetTickDataRes => Oa::ProtoOaGetTickdataRes);
request!(pb::ProtoOaDealListReq => Oa::ProtoOaDealListReq,
         pb::ProtoOaDealListRes => Oa::ProtoOaDealListRes);
request!(pb::ProtoOaTraderReq => Oa::ProtoOaTraderReq,
         pb::ProtoOaTraderRes => Oa::ProtoOaTraderRes);
request!(pb::ProtoOaGetAccountListByAccessTokenReq => Oa::ProtoOaGetAccountsByAccessTokenReq,
         pb::ProtoOaGetAccountListByAccessTokenRes => Oa::ProtoOaGetAccountsByAccessTokenRes);
request!(pb::ProtoOaAssetListReq => Oa::ProtoOaAssetListReq,
         pb::ProtoOaAssetListRes => Oa::ProtoOaAssetListRes);
request!(pb::ProtoOaReconcileReq => Oa::ProtoOaReconcileReq,
         pb::ProtoOaReconcileRes => Oa::ProtoOaReconcileRes);
request!(pb::ProtoOaGetPositionUnrealizedPnLReq => Oa::ProtoOaGetPositionUnrealizedPnlReq,
         pb::ProtoOaGetPositionUnrealizedPnLRes => Oa::ProtoOaGetPositionUnrealizedPnlRes);

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_not_impl_any;

    // Type level: no trading request implements `Request`, so none can be
    // passed to `Connection::request`. This fails to compile if one is added.
    assert_not_impl_any!(pb::ProtoOaNewOrderReq: Request);
    assert_not_impl_any!(pb::ProtoOaAmendPositionSltpReq: Request);
    assert_not_impl_any!(pb::ProtoOaClosePositionReq: Request);
    assert_not_impl_any!(pb::ProtoOaCancelOrderReq: Request);
    assert_not_impl_any!(pb::ProtoOaAmendOrderReq: Request);
    assert_not_impl_any!(pb::ProtoOaMarginCallUpdateReq: Request);

    /// The read requests, and nothing else.
    const EXPECTED_SENDABLE: [i32; 16] = [
        Common::HeartbeatEvent as i32,
        Oa::ProtoOaApplicationAuthReq as i32,
        Oa::ProtoOaAccountAuthReq as i32,
        Oa::ProtoOaRefreshTokenReq as i32,
        Oa::ProtoOaSymbolsListReq as i32,
        Oa::ProtoOaSymbolByIdReq as i32,
        Oa::ProtoOaGetTrendbarsReq as i32,
        Oa::ProtoOaSubscribeSpotsReq as i32,
        Oa::ProtoOaUnsubscribeSpotsReq as i32,
        Oa::ProtoOaGetTickdataReq as i32,
        Oa::ProtoOaDealListReq as i32,
        Oa::ProtoOaTraderReq as i32,
        Oa::ProtoOaGetAccountsByAccessTokenReq as i32,
        Oa::ProtoOaAssetListReq as i32,
        Oa::ProtoOaReconcileReq as i32,
        Oa::ProtoOaGetPositionUnrealizedPnlReq as i32,
    ];

    #[test]
    fn only_the_three_trading_requests_are_tradable() {
        let mut tradable = vec![];
        for t in 0..=3000u32 {
            if check_tradable(t).is_ok() {
                tradable.push(t);
            }
            // The two lists never overlap: a read path cannot send a trade.
            assert!(
                !(check_tradable(t).is_ok() && check_sendable(t).is_ok()),
                "{t}"
            );
        }
        assert_eq!(
            tradable,
            vec![
                Oa::ProtoOaNewOrderReq as u32,
                Oa::ProtoOaClosePositionReq as u32,
                Oa::ProtoOaAmendPositionSltpReq as u32,
            ]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn trading_payload_types_are_not_sendable() {
        let trading = [
            Oa::ProtoOaNewOrderReq,
            Oa::ProtoOaAmendPositionSltpReq,
            Oa::ProtoOaClosePositionReq,
            Oa::ProtoOaCancelOrderReq,
            Oa::ProtoOaAmendOrderReq,
            Oa::ProtoOaMarginCallUpdateReq,
        ];
        for t in trading {
            let err = check_sendable(t as u32).expect_err(t.as_str_name());
            assert_eq!(err.payload_type, t as u32);
            assert_eq!(err.name, t.as_str_name());
        }
    }

    #[test]
    fn every_payload_type_not_on_the_read_list_is_refused() {
        // Walk every payload type the protocol defines, plus the gaps between.
        let mut refused = 0;
        for t in 0..=3000u32 {
            let expected = EXPECTED_SENDABLE.contains(&(t as i32));
            assert_eq!(
                check_sendable(t).is_ok(),
                expected,
                "payload type {t} ({})",
                payload_name(t)
            );
            if !expected {
                refused += 1;
            }
        }
        assert_eq!(refused, 3001 - EXPECTED_SENDABLE.len());
        assert_eq!(SENDABLE.len(), EXPECTED_SENDABLE.len());
    }

    #[test]
    fn request_trait_payload_types_are_all_on_the_allowlist() {
        fn pt<R: Request>() -> u32 {
            R::PAYLOAD_TYPE
        }
        for t in [
            pt::<pb::ProtoOaApplicationAuthReq>(),
            pt::<pb::ProtoOaAccountAuthReq>(),
            pt::<pb::ProtoOaRefreshTokenReq>(),
            pt::<pb::ProtoOaSymbolsListReq>(),
            pt::<pb::ProtoOaSymbolByIdReq>(),
            pt::<pb::ProtoOaGetTrendbarsReq>(),
            pt::<pb::ProtoOaSubscribeSpotsReq>(),
            pt::<pb::ProtoOaUnsubscribeSpotsReq>(),
            pt::<pb::ProtoOaGetTickDataReq>(),
            pt::<pb::ProtoOaDealListReq>(),
            pt::<pb::ProtoOaTraderReq>(),
            pt::<pb::ProtoOaGetAccountListByAccessTokenReq>(),
            pt::<pb::ProtoOaAssetListReq>(),
            pt::<pb::ProtoOaReconcileReq>(),
            pt::<pb::ProtoOaGetPositionUnrealizedPnLReq>(),
        ] {
            assert!(check_sendable(t).is_ok(), "{t}");
        }
    }
}
