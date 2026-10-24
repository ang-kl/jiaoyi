//! A scripted, in-memory cTrader proxy for tests. It speaks the real framing
//! and the real protobuf messages over an in-memory pipe, so the client under
//! test runs exactly the code it runs against the broker.
//!
//! There are no broker credentials where this code is built; every frame the
//! fake sends is constructed from the .proto definitions, not recorded.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use prost::Message;

use crate::client::Connection;
use crate::frame::{read_frame, write_frame};
use crate::pb::{self, ProtoMessage, ProtoOaPayloadType as Oa, ProtoPayloadType as Common};

/// One scripted spot update: (bid, ask), either side may be absent.
pub type SidedQuote = (Option<u64>, Option<u64>);

/// What the fake broker serves.
#[derive(Debug, Clone, Default)]
pub struct FakeBroker {
    /// The one account this fake knows. Others get CH_CTID_TRADER_ACCOUNT_NOT_FOUND.
    pub account_id: i64,
    /// Access tokens the fake treats as expired (OA_AUTH_TOKEN_EXPIRED).
    pub expired_tokens: HashSet<String>,
    /// Reply to a refresh: (access, refresh, expires_in seconds).
    pub refresh_reply: Option<(String, String, i64)>,
    pub light_symbols: Vec<pb::ProtoOaLightSymbol>,
    pub symbols: Vec<pb::ProtoOaSymbol>,
    /// D1 bars per symbol id.
    pub bars: HashMap<i64, Vec<pb::ProtoOaTrendbar>>,
    /// Quotes (bid, ask) sent per symbol id after a spot subscription.
    pub quotes: HashMap<i64, Vec<SidedQuote>>,
    /// At most this many bars per trendbar response (0 = unlimited); more sets hasMore.
    pub max_bars_per_response: usize,
}

/// What the fake saw.
#[derive(Debug, Clone, Default)]
pub struct FakeLog {
    pub payload_types: Vec<u32>,
    pub trendbar_requests: Vec<(i64, i64, i64)>,
    pub account_auths: Vec<i64>,
}

fn reply(id: Option<String>, pt: i32, payload: Vec<u8>) -> ProtoMessage {
    ProtoMessage {
        payload_type: pt as u32,
        payload: Some(payload),
        client_msg_id: id,
    }
}

fn oa_error(id: Option<String>, code: &str) -> ProtoMessage {
    reply(
        id,
        Oa::ProtoOaErrorRes as i32,
        pb::ProtoOaErrorRes {
            error_code: code.into(),
            description: Some(format!("fake broker: {code}")),
            ..Default::default()
        }
        .encode_to_vec(),
    )
}

impl FakeBroker {
    /// Start the fake and return a client connection to it, plus its log.
    pub fn spawn(self) -> (Connection, Arc<Mutex<FakeLog>>) {
        let (client_side, mut server) = tokio::io::duplex(1 << 22);
        let log = Arc::new(Mutex::new(FakeLog::default()));
        let log2 = log.clone();
        tokio::spawn(async move {
            while let Ok(msg) = read_frame(&mut server).await {
                let id = msg.client_msg_id.clone();
                let body = msg.payload.clone().unwrap_or_default();
                log2.lock().unwrap().payload_types.push(msg.payload_type);
                let mut out: Vec<ProtoMessage> = Vec::new();
                let pt = msg.payload_type as i32;
                if pt == Common::HeartbeatEvent as i32 {
                    continue;
                } else if pt == Oa::ProtoOaApplicationAuthReq as i32 {
                    out.push(reply(id, Oa::ProtoOaApplicationAuthRes as i32, vec![]));
                } else if pt == Oa::ProtoOaAccountAuthReq as i32 {
                    let req = pb::ProtoOaAccountAuthReq::decode(body.as_slice()).unwrap();
                    log2.lock()
                        .unwrap()
                        .account_auths
                        .push(req.ctid_trader_account_id);
                    if req.ctid_trader_account_id != self.account_id {
                        out.push(oa_error(id, "CH_CTID_TRADER_ACCOUNT_NOT_FOUND"));
                    } else if self.expired_tokens.contains(&req.access_token) {
                        out.push(oa_error(id, "OA_AUTH_TOKEN_EXPIRED"));
                    } else {
                        out.push(reply(
                            id,
                            Oa::ProtoOaAccountAuthRes as i32,
                            pb::ProtoOaAccountAuthRes {
                                payload_type: None,
                                ctid_trader_account_id: req.ctid_trader_account_id,
                            }
                            .encode_to_vec(),
                        ));
                    }
                } else if pt == Oa::ProtoOaRefreshTokenReq as i32 {
                    match &self.refresh_reply {
                        Some((a, r, e)) => out.push(reply(
                            id,
                            Oa::ProtoOaRefreshTokenRes as i32,
                            pb::ProtoOaRefreshTokenRes {
                                payload_type: None,
                                access_token: a.clone(),
                                token_type: "bearer".into(),
                                expires_in: *e,
                                refresh_token: r.clone(),
                            }
                            .encode_to_vec(),
                        )),
                        None => out.push(oa_error(id, "CH_ACCESS_TOKEN_INVALID")),
                    }
                } else if pt == Oa::ProtoOaSymbolsListReq as i32 {
                    out.push(reply(
                        id,
                        Oa::ProtoOaSymbolsListRes as i32,
                        pb::ProtoOaSymbolsListRes {
                            payload_type: None,
                            ctid_trader_account_id: self.account_id,
                            symbol: self.light_symbols.clone(),
                            archived_symbol: vec![],
                        }
                        .encode_to_vec(),
                    ));
                } else if pt == Oa::ProtoOaSymbolByIdReq as i32 {
                    let req = pb::ProtoOaSymbolByIdReq::decode(body.as_slice()).unwrap();
                    out.push(reply(
                        id,
                        Oa::ProtoOaSymbolByIdRes as i32,
                        pb::ProtoOaSymbolByIdRes {
                            payload_type: None,
                            ctid_trader_account_id: self.account_id,
                            symbol: self
                                .symbols
                                .iter()
                                .filter(|s| req.symbol_id.contains(&s.symbol_id))
                                .cloned()
                                .collect(),
                            archived_symbol: vec![],
                        }
                        .encode_to_vec(),
                    ));
                } else if pt == Oa::ProtoOaGetTrendbarsReq as i32 {
                    let req = pb::ProtoOaGetTrendbarsReq::decode(body.as_slice()).unwrap();
                    let from = req.from_timestamp.unwrap_or(0);
                    let to = req.to_timestamp.unwrap_or(i64::MAX);
                    log2.lock()
                        .unwrap()
                        .trendbar_requests
                        .push((req.symbol_id, from, to));
                    let mut bars: Vec<_> = self
                        .bars
                        .get(&req.symbol_id)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|b| {
                            let ms = i64::from(b.utc_timestamp_in_minutes.unwrap_or(0)) * 60_000;
                            ms >= from && ms <= to
                        })
                        .collect();
                    let mut has_more = false;
                    if self.max_bars_per_response > 0 && bars.len() > self.max_bars_per_response {
                        // Newest first, back from toTimestamp, like `count`.
                        bars.sort_by_key(|b| b.utc_timestamp_in_minutes);
                        bars = bars.split_off(bars.len() - self.max_bars_per_response);
                        has_more = true;
                    }
                    out.push(reply(
                        id,
                        Oa::ProtoOaGetTrendbarsRes as i32,
                        pb::ProtoOaGetTrendbarsRes {
                            payload_type: None,
                            ctid_trader_account_id: self.account_id,
                            period: pb::ProtoOaTrendbarPeriod::D1 as i32,
                            trendbar: bars,
                            symbol_id: Some(req.symbol_id),
                            has_more: Some(has_more),
                            ..Default::default()
                        }
                        .encode_to_vec(),
                    ));
                } else if pt == Oa::ProtoOaSubscribeSpotsReq as i32 {
                    let req = pb::ProtoOaSubscribeSpotsReq::decode(body.as_slice()).unwrap();
                    out.push(reply(
                        id,
                        Oa::ProtoOaSubscribeSpotsRes as i32,
                        pb::ProtoOaSubscribeSpotsRes {
                            payload_type: None,
                            ctid_trader_account_id: self.account_id,
                        }
                        .encode_to_vec(),
                    ));
                    for sid in &req.symbol_id {
                        for (bid, ask) in self.quotes.get(sid).cloned().unwrap_or_default() {
                            out.push(reply(
                                None,
                                Oa::ProtoOaSpotEvent as i32,
                                pb::ProtoOaSpotEvent {
                                    ctid_trader_account_id: self.account_id,
                                    symbol_id: *sid,
                                    bid,
                                    ask,
                                    ..Default::default()
                                }
                                .encode_to_vec(),
                            ));
                        }
                    }
                } else if pt == Oa::ProtoOaUnsubscribeSpotsReq as i32 {
                    out.push(reply(
                        id,
                        Oa::ProtoOaUnsubscribeSpotsRes as i32,
                        pb::ProtoOaUnsubscribeSpotsRes {
                            payload_type: None,
                            ctid_trader_account_id: self.account_id,
                        }
                        .encode_to_vec(),
                    ));
                } else {
                    out.push(oa_error(id, "UNSUPPORTED_MESSAGE"));
                }
                for m in out {
                    if write_frame(&mut server, &m).await.is_err() {
                        return;
                    }
                }
            }
        });
        (
            Connection::from_stream(client_side, Duration::from_secs(5)),
            log,
        )
    }
}

/// A D1 trendbar from exact relative prices (1/100000) at a UTC day start.
pub fn d1_bar(
    day: chrono::NaiveDate,
    open: i64,
    high: i64,
    low: i64,
    close: i64,
    volume: i64,
) -> pb::ProtoOaTrendbar {
    let minutes = day.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() / 60;
    pb::ProtoOaTrendbar {
        volume,
        period: Some(pb::ProtoOaTrendbarPeriod::D1 as i32),
        low: Some(low),
        delta_open: Some((open - low) as u64),
        delta_close: Some((close - low) as u64),
        delta_high: Some((high - low) as u64),
        utc_timestamp_in_minutes: Some(minutes as u32),
    }
}
