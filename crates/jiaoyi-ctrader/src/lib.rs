//! cTrader Open API client.
//!
//! Phase 1 uses it to read: application and account authorisation, the symbol
//! list, symbol details, daily trendbars, spot quotes (for spread sampling) and
//! token refresh.
//!
//! Every frame this program writes passes [`payload::check_sendable`], the one
//! place that names the request payload types the program may send. A request
//! whose type is not listed there is refused before a byte is written.
//!
//! The generated protobuf types live in a private module. Only the message
//! types phase 1 reads or sends are re-exported, in [`messages`]; the order,
//! amend, close and cancel request types cannot be named outside this crate.

mod pb {
    #![allow(clippy::all, missing_docs)]
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

pub mod client;
pub mod frame;
pub mod payload;
pub mod trendbar;

#[cfg(feature = "testkit")]
pub mod testkit;

/// The protobuf messages phase 1 may build or read.
pub mod messages {
    pub use crate::pb::{
        ProtoErrorRes, ProtoHeartbeatEvent, ProtoMessage, ProtoOaAccountAuthReq,
        ProtoOaAccountAuthRes, ProtoOaAccountDisconnectEvent, ProtoOaAccountsTokenInvalidatedEvent,
        ProtoOaApplicationAuthReq, ProtoOaApplicationAuthRes, ProtoOaArchivedSymbol,
        ProtoOaClientDisconnectEvent, ProtoOaCommissionType, ProtoOaDayOfWeek, ProtoOaErrorRes,
        ProtoOaGetTrendbarsReq, ProtoOaGetTrendbarsRes, ProtoOaInterval, ProtoOaLightSymbol,
        ProtoOaMinCommissionType, ProtoOaPayloadType, ProtoOaRefreshTokenReq,
        ProtoOaRefreshTokenRes, ProtoOaSpotEvent, ProtoOaSubscribeSpotsReq,
        ProtoOaSubscribeSpotsRes, ProtoOaSwapCalculationType, ProtoOaSymbol, ProtoOaSymbolByIdReq,
        ProtoOaSymbolByIdRes, ProtoOaSymbolsListReq, ProtoOaSymbolsListRes, ProtoOaTradingMode,
        ProtoOaTrendbar, ProtoOaTrendbarPeriod, ProtoOaUnsubscribeSpotsReq,
        ProtoOaUnsubscribeSpotsRes, ProtoPayloadType,
    };
}

pub use client::{
    AccountSession, ClientError, Connection, ReceivedSpot, RefreshedToken, SessionEnd,
};
pub use payload::{check_sendable, NotSendable};
pub use trendbar::{closed_daily_bars, decode_trendbar, DailyBar, TrendbarError};
