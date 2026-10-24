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
    /// Refuse this many account authorisations with CH_ACCESS_TOKEN_INVALID,
    /// whatever the token. Shared by clones, so it spans connections.
    pub auth_rejections: Arc<std::sync::atomic::AtomicUsize>,
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
    /// Close the socket right after sending the quotes that follow a spot
    /// subscription, as a broker disconnect would.
    pub close_after_spots: bool,
    /// Send `ProtoOAAccountsTokenInvalidatedEvent` for the account right after
    /// the quotes that follow a spot subscription, keeping the socket open (as
    /// a token refresh elsewhere does).
    pub invalidate_after_spots: bool,
    /// Historical ticks per (symbol id, `ProtoOAQuoteType` 1 bid / 2 ask):
    /// (timestamp ms, price e5), in any order.
    pub tick_history: HashMap<(i64, i32), Vec<(i64, i64)>>,
    /// At most this many ticks per response (0 = unlimited); more sets hasMore.
    pub max_ticks_per_response: usize,
    /// Refuse every tick request whose window starts before this time (ms)
    /// with BLOCKED_PAYLOAD_TYPE and retryAfter 0, as a rate limit would.
    pub block_ticks_before: Option<i64>,
    /// Deal history of the account.
    pub deals: Vec<pb::ProtoOaDeal>,
    /// At most this many deals per response (0 = unlimited); more sets hasMore.
    pub max_deals_per_response: usize,
    /// The trader record; a default one when `None`.
    pub trader: Option<pb::ProtoOaTrader>,
    /// The token's grant (`ProtoOAGetAccountListByAccessTokenRes`). `None`:
    /// the fake's own account, demo. An account the grant omits is refused
    /// authorisation with CH_ACCESS_TOKEN_INVALID, as the broker does.
    pub grant: Option<Vec<GrantEntry>>,
    /// A grant for one access token, overriding `grant` for that token.
    pub grant_by_token: HashMap<String, Vec<GrantEntry>>,
    /// Refresh requests received, shared by clones (spans connections).
    pub refresh_count: Arc<std::sync::atomic::AtomicUsize>,
    /// A token event hub shared by fakes: every refresh on any of them sends
    /// ProtoOAAccountsTokenInvalidatedEvent for its account on all of them, as
    /// a refresh does to every session using the old token.
    pub token_events: Option<tokio::sync::broadcast::Sender<()>>,
    /// Count requests of this payload type down (the counter is shared by
    /// clones, so it spans connections); the request that reaches 0 gets
    /// ProtoOAAccountsTokenInvalidatedEvent and ACCOUNT_NOT_AUTHORIZED, and
    /// every later account request on that connection ACCOUNT_NOT_AUTHORIZED
    /// until the account is authorised again.
    pub invalidate_on: Option<(i32, Arc<std::sync::atomic::AtomicUsize>)>,
}

/// One account in a token's grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantEntry {
    pub ctid: i64,
    pub is_live: Option<bool>,
    pub login: Option<i64>,
}

fn token_invalidated(account_id: i64) -> ProtoMessage {
    reply(
        None,
        Oa::ProtoOaAccountsTokenInvalidatedEvent as i32,
        pb::ProtoOaAccountsTokenInvalidatedEvent {
            payload_type: None,
            ctid_trader_account_ids: vec![account_id],
            reason: Some("Access Token is expired or recalled".into()),
        }
        .encode_to_vec(),
    )
}

/// What the fake saw.
#[derive(Debug, Clone, Default)]
pub struct FakeLog {
    pub payload_types: Vec<u32>,
    pub trendbar_requests: Vec<(i64, i64, i64)>,
    pub tick_requests: Vec<(i64, i32, i64, i64)>,
    pub deal_requests: Vec<(i64, i64)>,
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
    fn grant_entries(&self, token: &str) -> Vec<GrantEntry> {
        if let Some(g) = self.grant_by_token.get(token) {
            return g.clone();
        }
        self.grant.clone().unwrap_or_else(|| {
            vec![GrantEntry {
                ctid: self.account_id,
                is_live: Some(false),
                login: None,
            }]
        })
    }

    pub fn spawn(self) -> (Connection, Arc<Mutex<FakeLog>>) {
        use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
        let (client_side, server) = tokio::io::duplex(1 << 22);
        let (mut rd, wr) = tokio::io::split(server);
        let wr = Arc::new(tokio::sync::Mutex::new(wr));
        let log = Arc::new(Mutex::new(FakeLog::default()));
        let log2 = log.clone();
        let invalidated = Arc::new(AtomicBool::new(false));
        let events_task = self.token_events.as_ref().map(|tx| {
            let mut rx = tx.subscribe();
            let (wr, inv, account) = (wr.clone(), invalidated.clone(), self.account_id);
            tokio::spawn(async move {
                while rx.recv().await.is_ok() {
                    inv.store(true, SeqCst);
                    if write_frame(&mut *wr.lock().await, &token_invalidated(account))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            })
        });
        tokio::spawn(async move {
            let serve = async {
                while let Ok(msg) = read_frame(&mut rd).await {
                    let id = msg.client_msg_id.clone();
                    let body = msg.payload.clone().unwrap_or_default();
                    log2.lock().unwrap().payload_types.push(msg.payload_type);
                    let mut out: Vec<ProtoMessage> = Vec::new();
                    let pt = msg.payload_type as i32;
                    let session_free = pt == Oa::ProtoOaApplicationAuthReq as i32
                        || pt == Oa::ProtoOaAccountAuthReq as i32
                        || pt == Oa::ProtoOaRefreshTokenReq as i32
                        || pt == Oa::ProtoOaGetAccountsByAccessTokenReq as i32;
                    if pt == Common::HeartbeatEvent as i32 {
                        continue;
                    } else if invalidated.load(SeqCst) && !session_free {
                        out.push(oa_error(id, "ACCOUNT_NOT_AUTHORIZED"));
                    } else if self.invalidate_on.as_ref().is_some_and(|(t, c)| {
                        *t == pt && c.fetch_update(SeqCst, SeqCst, |n| n.checked_sub(1)) == Ok(1)
                    }) {
                        // The broker ends the session: the event, then the
                        // request's own refusal.
                        invalidated.store(true, SeqCst);
                        out.push(token_invalidated(self.account_id));
                        out.push(oa_error(id, "ACCOUNT_NOT_AUTHORIZED"));
                    } else if pt == Oa::ProtoOaGetAccountsByAccessTokenReq as i32 {
                        let req =
                            pb::ProtoOaGetAccountListByAccessTokenReq::decode(body.as_slice())
                                .unwrap();
                        if self.expired_tokens.contains(&req.access_token) {
                            out.push(oa_error(id, "CH_ACCESS_TOKEN_INVALID"));
                        } else {
                            let entries = self.grant_entries(&req.access_token);
                            out.push(reply(
                                id,
                                Oa::ProtoOaGetAccountsByAccessTokenRes as i32,
                                pb::ProtoOaGetAccountListByAccessTokenRes {
                                    payload_type: None,
                                    access_token: req.access_token,
                                    permission_scope: Some(1),
                                    ctid_trader_account: entries
                                        .into_iter()
                                        .map(|g| pb::ProtoOaCtidTraderAccount {
                                            ctid_trader_account_id: g.ctid as u64,
                                            is_live: g.is_live,
                                            trader_login: g.login,
                                            ..Default::default()
                                        })
                                        .collect(),
                                }
                                .encode_to_vec(),
                            ));
                        }
                    } else if pt == Oa::ProtoOaApplicationAuthReq as i32 {
                        out.push(reply(id, Oa::ProtoOaApplicationAuthRes as i32, vec![]));
                    } else if pt == Oa::ProtoOaAccountAuthReq as i32 {
                        let req = pb::ProtoOaAccountAuthReq::decode(body.as_slice()).unwrap();
                        log2.lock()
                            .unwrap()
                            .account_auths
                            .push(req.ctid_trader_account_id);
                        let reject = self
                            .auth_rejections
                            .fetch_update(
                                std::sync::atomic::Ordering::SeqCst,
                                std::sync::atomic::Ordering::SeqCst,
                                |n| n.checked_sub(1),
                            )
                            .is_ok();
                        if reject {
                            out.push(oa_error(id, "CH_ACCESS_TOKEN_INVALID"));
                        } else if req.ctid_trader_account_id != self.account_id {
                            out.push(oa_error(id, "CH_CTID_TRADER_ACCOUNT_NOT_FOUND"));
                        } else if self.expired_tokens.contains(&req.access_token) {
                            out.push(oa_error(id, "OA_AUTH_TOKEN_EXPIRED"));
                        } else if !self
                            .grant_entries(&req.access_token)
                            .iter()
                            .any(|g| g.ctid == req.ctid_trader_account_id)
                        {
                            // What the broker does for an account the grant omits.
                            out.push(oa_error(id, "CH_ACCESS_TOKEN_INVALID"));
                        } else {
                            invalidated.store(false, SeqCst);
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
                        self.refresh_count.fetch_add(1, SeqCst);
                        if let Some(tx) = &self.token_events {
                            let _ = tx.send(());
                        }
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
                                let ms =
                                    i64::from(b.utc_timestamp_in_minutes.unwrap_or(0)) * 60_000;
                                ms >= from && ms <= to
                            })
                            .collect();
                        let mut has_more = false;
                        if self.max_bars_per_response > 0 && bars.len() > self.max_bars_per_response
                        {
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
                    } else if pt == Oa::ProtoOaGetTickdataReq as i32 {
                        let req = pb::ProtoOaGetTickDataReq::decode(body.as_slice()).unwrap();
                        let (from, to) = (
                            req.from_timestamp.unwrap_or(0),
                            req.to_timestamp.unwrap_or(i64::MAX),
                        );
                        log2.lock().unwrap().tick_requests.push((
                            req.symbol_id,
                            req.r#type,
                            from,
                            to,
                        ));
                        if self.block_ticks_before.is_some_and(|b| from < b) {
                            out.push(reply(
                                id,
                                Oa::ProtoOaErrorRes as i32,
                                pb::ProtoOaErrorRes {
                                    error_code: "BLOCKED_PAYLOAD_TYPE".into(),
                                    description: Some("fake broker: rate limit".into()),
                                    retry_after: Some(0),
                                    ..Default::default()
                                }
                                .encode_to_vec(),
                            ));
                            for m in out {
                                if write_frame(&mut *wr.lock().await, &m).await.is_err() {
                                    return;
                                }
                            }
                            continue;
                        }
                        let mut ticks: Vec<(i64, i64)> = self
                            .tick_history
                            .get(&(req.symbol_id, req.r#type))
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|(ts, _)| *ts >= from && *ts <= to)
                            .collect();
                        // Newest first; a stable sort keeps the scripted order of
                        // ticks that share a millisecond.
                        ticks.sort_by(|a, b| b.0.cmp(&a.0));
                        let mut has_more = false;
                        if self.max_ticks_per_response > 0
                            && ticks.len() > self.max_ticks_per_response
                        {
                            ticks.truncate(self.max_ticks_per_response);
                            has_more = true;
                        }
                        // First absolute, then differences from the previous tick.
                        let mut data = Vec::with_capacity(ticks.len());
                        let mut prev: Option<(i64, i64)> = None;
                        for (ts, p) in ticks {
                            data.push(match prev {
                                None => pb::ProtoOaTickData {
                                    timestamp: ts,
                                    tick: p,
                                },
                                Some((pts, pp)) => pb::ProtoOaTickData {
                                    timestamp: ts - pts,
                                    tick: p - pp,
                                },
                            });
                            prev = Some((ts, p));
                        }
                        out.push(reply(
                            id,
                            Oa::ProtoOaGetTickdataRes as i32,
                            pb::ProtoOaGetTickDataRes {
                                payload_type: None,
                                ctid_trader_account_id: self.account_id,
                                tick_data: data,
                                has_more,
                            }
                            .encode_to_vec(),
                        ));
                    } else if pt == Oa::ProtoOaDealListReq as i32 {
                        let req = pb::ProtoOaDealListReq::decode(body.as_slice()).unwrap();
                        let (from, to) = (
                            req.from_timestamp.unwrap_or(0),
                            req.to_timestamp.unwrap_or(i64::MAX),
                        );
                        log2.lock().unwrap().deal_requests.push((from, to));
                        let mut deals: Vec<pb::ProtoOaDeal> = self
                            .deals
                            .iter()
                            .filter(|d| {
                                d.execution_timestamp >= from && d.execution_timestamp <= to
                            })
                            .cloned()
                            .collect();
                        let mut has_more = false;
                        if self.max_deals_per_response > 0
                            && deals.len() > self.max_deals_per_response
                        {
                            deals.truncate(self.max_deals_per_response);
                            has_more = true;
                        }
                        out.push(reply(
                            id,
                            Oa::ProtoOaDealListRes as i32,
                            pb::ProtoOaDealListRes {
                                payload_type: None,
                                ctid_trader_account_id: self.account_id,
                                deal: deals,
                                has_more,
                            }
                            .encode_to_vec(),
                        ));
                    } else if pt == Oa::ProtoOaTraderReq as i32 {
                        let trader = self.trader.clone().unwrap_or(pb::ProtoOaTrader {
                            ctid_trader_account_id: self.account_id,
                            balance: 1_000_000,
                            deposit_asset_id: 1,
                            money_digits: Some(2),
                            ..Default::default()
                        });
                        out.push(reply(
                            id,
                            Oa::ProtoOaTraderRes as i32,
                            pb::ProtoOaTraderRes {
                                payload_type: None,
                                ctid_trader_account_id: self.account_id,
                                trader,
                            }
                            .encode_to_vec(),
                        ));
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
                        if write_frame(&mut *wr.lock().await, &m).await.is_err() {
                            return;
                        }
                    }
                    if self.invalidate_after_spots && pt == Oa::ProtoOaSubscribeSpotsReq as i32 {
                        let ev = token_invalidated(self.account_id);
                        if write_frame(&mut *wr.lock().await, &ev).await.is_err() {
                            return;
                        }
                    }
                    if self.close_after_spots && pt == Oa::ProtoOaSubscribeSpotsReq as i32 {
                        // Dropping the server halves closes the client's read side.
                        return;
                    }
                }
            };
            serve.await;
            if let Some(t) = events_task {
                t.abort();
            }
            drop(wr);
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
