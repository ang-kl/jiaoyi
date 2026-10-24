//! The connection to a cTrader Open API proxy.
//!
//! One background task reads frames and routes them: a response goes to the
//! request waiting on its `clientMsgId`; a spot event goes to the spot
//! broadcast; disconnect and token-invalidated events are logged and kept.
//! Another task sends a heartbeat every 10 seconds (the proxy expects one
//! within 30 s of silence; `ProtoHeartbeatEvent` docs).
//!
//! Every outgoing frame goes through [`Connection::send_message`], which calls
//! [`check_sendable`] before writing. There is no other write path.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::PermittedAccount;
use jiaoyi_core::secret::Secret;
use prost::Message;
use rust_decimal::Decimal;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{broadcast, oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::Instant;

use crate::frame::{read_frame, write_frame, FrameError};
use crate::payload::{check_sendable, payload_name, NotSendable, Request, Response};
use crate::pb::{
    self, ProtoErrorRes, ProtoMessage, ProtoOaErrorRes, ProtoOaPayloadType as Oa, ProtoOaSpotEvent,
    ProtoOaTrendbar, ProtoOaTrendbarPeriod, ProtoPayloadType as Common,
};

/// The proxy drops a connection after 30 s of silence; send well inside that.
pub const HEARTBEAT_EVERY: Duration = Duration::from_secs(10);
/// Spotware limits historical-data requests to 5 per second; stay under it.
pub const HISTORICAL_MIN_GAP: Duration = Duration::from_millis(250);
/// Longest D1 window asked for in one request (one year plus a day).
pub const D1_WINDOW_DAYS: i64 = 366;
/// Stop walking back through history after this many empty windows in a row.
pub const EMPTY_WINDOWS_TO_STOP: u32 = 3;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    NotSendable(#[from] NotSendable),
    #[error("frame: {0}")]
    Frame(#[from] FrameError),
    #[error("connect: {0}")]
    Connect(String),
    #[error("broker error {code}: {}", description.as_deref().unwrap_or(""))]
    Broker {
        code: String,
        description: Option<String>,
        retry_after_secs: Option<u64>,
    },
    #[error("expected payload type {expected} ({}), got {got} ({})", payload_name(*expected), payload_name(*got))]
    UnexpectedPayload { expected: u32, got: u32 },
    #[error("response does not decode: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("no response within {0:?}")]
    Timeout(Duration),
    #[error("connection closed: {0}")]
    Disconnected(String),
    #[error("response is for account {got}, asked for {asked}")]
    WrongAccount { asked: i64, got: i64 },
    #[error("tick data: {0}")]
    TickData(#[from] crate::tickdata::TickDataError),
}

impl ClientError {
    /// True when the broker says the access token is expired or invalid.
    pub fn is_token_rejected(&self) -> bool {
        matches!(self, ClientError::Broker { code, .. }
            if code == "OA_AUTH_TOKEN_EXPIRED" || code == "CH_ACCESS_TOKEN_INVALID")
    }
}

type Pending = Arc<StdMutex<HashMap<String, oneshot::Sender<ProtoMessage>>>>;

/// The broker ended one or more account sessions on a socket that stays open
/// (`ProtoOAAccountsTokenInvalidatedEvent`, `ProtoOAAccountDisconnectEvent`,
/// or `ProtoOAClientDisconnectEvent` for every account). A session for an
/// affected account must re-authorise before it can be used again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEnd {
    /// Empty: every account on this connection.
    pub account_ids: Vec<i64>,
    pub reason: String,
}

impl SessionEnd {
    pub fn affects(&self, account_id: i64) -> bool {
        self.account_ids.is_empty() || self.account_ids.contains(&account_id)
    }
}

/// A spot event and the moment the reader task decoded it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceivedSpot {
    pub received_at: DateTime<Utc>,
    pub event: ProtoOaSpotEvent,
}
type Writer = Box<dyn AsyncWrite + Send + Unpin>;

/// A connection to one cTrader proxy (demo or live).
pub struct Connection {
    writer: Arc<Mutex<Writer>>,
    pending: Pending,
    next_id: AtomicU64,
    /// The spot broadcast. Taken (set to `None`) when the connection closes, so
    /// that every spot consumer's `recv` returns `Closed` instead of waiting
    /// for events that can no longer arrive.
    spots: Arc<StdMutex<Option<broadcast::Sender<ReceivedSpot>>>>,
    notices: Arc<StdMutex<Vec<String>>>,
    /// Account sessions the broker ended, in order, and their broadcast.
    session_ends: Arc<StdMutex<Vec<SessionEnd>>>,
    session_end_tx: broadcast::Sender<SessionEnd>,
    closed: Arc<StdMutex<Option<String>>>,
    historical_gate: Mutex<Option<Instant>>,
    request_timeout: Duration,
    tasks: Vec<JoinHandle<()>>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        for t in &self.tasks {
            t.abort();
        }
    }
}

/// One asset of an account, as `ProtoOAAsset` gives it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Asset {
    pub asset_id: i64,
    pub name: String,
    pub display_name: Option<String>,
    pub digits: Option<i32>,
}

/// The accounts an access token's grant covers, as
/// `ProtoOAGetAccountListByAccessTokenRes` gives them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenGrant {
    /// `SCOPE_VIEW` or `SCOPE_TRADE`, as the broker names it; `None` when not sent.
    pub permission_scope: Option<String>,
    pub accounts: Vec<GrantedAccount>,
}

/// One account in a token's grant.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GrantedAccount {
    pub ctid: i64,
    /// `None` when the broker did not say.
    pub is_live: Option<bool>,
    pub trader_login: Option<i64>,
}

impl TokenGrant {
    pub fn account(&self, ctid: i64) -> Option<&GrantedAccount> {
        self.accounts.iter().find(|a| a.ctid == ctid)
    }
}

/// A refreshed OAuth token, as `ProtoOARefreshTokenRes` gives it.
#[derive(Debug, Clone)]
pub struct RefreshedToken {
    pub access_token: Secret,
    pub refresh_token: Secret,
    pub token_type: String,
    pub expires_in_secs: i64,
}

impl Connection {
    /// Open a TLS connection to `host:port` (e.g. demo.ctraderapi.com:5035).
    pub async fn connect_tls(host: &str, port: u16) -> Result<Connection, ClientError> {
        let mut roots = rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(|e| ClientError::Connect(e.to_string()))?
        .with_root_certificates(roots)
        .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
        let tcp = tokio::net::TcpStream::connect((host, port))
            .await
            .map_err(|e| ClientError::Connect(format!("{host}:{port}: {e}")))?;
        let name = rustls::pki_types::ServerName::try_from(host.to_string())
            .map_err(|e| ClientError::Connect(format!("{host}: {e}")))?;
        let tls = connector
            .connect(name, tcp)
            .await
            .map_err(|e| ClientError::Connect(format!("TLS {host}:{port}: {e}")))?;
        Ok(Connection::from_stream(tls, Duration::from_secs(30)))
    }

    /// Wrap any byte stream (TLS in production, an in-memory pipe in tests).
    pub fn from_stream<S>(stream: S, request_timeout: Duration) -> Connection
    where
        S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        let (mut reader, writer) = tokio::io::split(stream);
        let writer: Arc<Mutex<Writer>> = Arc::new(Mutex::new(Box::new(writer) as Writer));
        let pending: Pending = Arc::default();
        let (spots_tx, _) = broadcast::channel(4096);
        let spots = Arc::new(StdMutex::new(Some(spots_tx.clone())));
        let notices: Arc<StdMutex<Vec<String>>> = Arc::default();
        let session_ends: Arc<StdMutex<Vec<SessionEnd>>> = Arc::default();
        let (session_end_tx, _) = broadcast::channel(64);
        let closed: Arc<StdMutex<Option<String>>> = Arc::default();

        let read_task = {
            let pending = pending.clone();
            let spots_slot = spots.clone();
            let notices = notices.clone();
            let ends = (session_ends.clone(), session_end_tx.clone());
            let closed = closed.clone();
            tokio::spawn(async move {
                let reason = loop {
                    match read_frame(&mut reader).await {
                        Ok(msg) => route(msg, &pending, &spots_tx, &notices, &ends),
                        Err(e) => break e.to_string(),
                    }
                };
                tracing::warn!(%reason, "cTrader connection closed");
                *closed.lock().unwrap() = Some(reason);
                // Dropping the senders wakes every waiting request with an error.
                pending.lock().unwrap().clear();
                // Drop every spot sender: consumers see `Closed` once drained.
                spots_slot.lock().unwrap().take();
                drop(spots_tx);
            })
        };

        let mut conn = Connection {
            writer,
            pending,
            next_id: AtomicU64::new(1),
            spots,
            notices,
            session_ends,
            session_end_tx,
            closed,
            historical_gate: Mutex::new(None),
            request_timeout,
            tasks: vec![read_task],
        };

        let heartbeat_task = {
            let writer = conn.writer.clone();
            tokio::spawn(async move {
                let mut tick = tokio::time::interval(HEARTBEAT_EVERY);
                tick.tick().await;
                loop {
                    tick.tick().await;
                    let hb = pb::ProtoHeartbeatEvent::default().encode_to_vec();
                    if send_checked(&writer, Common::HeartbeatEvent as u32, hb, None)
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            })
        };
        conn.tasks.push(heartbeat_task);
        conn
    }

    /// The single write path. Refuses any payload type not on the allowlist.
    async fn send_message(
        &self,
        payload_type: u32,
        payload: Vec<u8>,
        client_msg_id: Option<String>,
    ) -> Result<(), ClientError> {
        send_checked(&self.writer, payload_type, payload, client_msg_id).await
    }

    /// Send a request and wait for its response.
    pub(crate) async fn request<R: Request>(&self, req: &R) -> Result<R::Response, ClientError> {
        if let Some(reason) = self.closed.lock().unwrap().clone() {
            return Err(ClientError::Disconnected(reason));
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed).to_string();
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id.clone(), tx);
        if let Err(e) = self
            .send_message(R::PAYLOAD_TYPE, req.encode_to_vec(), Some(id.clone()))
            .await
        {
            self.pending.lock().unwrap().remove(&id);
            return Err(e);
        }
        let msg = match tokio::time::timeout(self.request_timeout, rx).await {
            Ok(Ok(msg)) => msg,
            Ok(Err(_)) => {
                let reason = self.closed.lock().unwrap().clone();
                return Err(ClientError::Disconnected(
                    reason.unwrap_or_else(|| "reader stopped".into()),
                ));
            }
            Err(_) => {
                self.pending.lock().unwrap().remove(&id);
                return Err(ClientError::Timeout(self.request_timeout));
            }
        };
        decode_response::<R::Response>(msg)
    }

    /// A receiver of spot events. Subscribe before asking for spots. On a
    /// closed connection the receiver is already closed.
    pub fn spot_events(&self) -> broadcast::Receiver<ReceivedSpot> {
        match self.spots.lock().unwrap().as_ref() {
            Some(tx) => tx.subscribe(),
            None => broadcast::channel(1).1,
        }
    }

    /// A receiver of account-session ends. Take it before authorising, then
    /// also check [`Connection::session_ended`] for an end that came earlier.
    pub fn session_ends(&self) -> broadcast::Receiver<SessionEnd> {
        self.session_end_tx.subscribe()
    }

    /// The first recorded end affecting `account_id` on this connection, if any.
    pub fn session_ended(&self, account_id: i64) -> Option<SessionEnd> {
        self.session_ends
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.affects(account_id))
            .cloned()
    }

    /// Why the connection closed, once it has.
    pub fn closed_reason(&self) -> Option<String> {
        self.closed.lock().unwrap().clone()
    }

    /// Disconnect and token-invalidated notices received so far.
    pub fn notices(&self) -> Vec<String> {
        self.notices.lock().unwrap().clone()
    }

    pub async fn application_auth(
        &self,
        client_id: &str,
        client_secret: &Secret,
    ) -> Result<(), ClientError> {
        self.request(&pb::ProtoOaApplicationAuthReq {
            payload_type: None,
            client_id: client_id.to_string(),
            client_secret: client_secret.expose().to_string(),
        })
        .await?;
        Ok(())
    }

    /// Exchange a refresh token for a new access and refresh token. The old
    /// refresh token stops working; the caller must store the new one.
    pub async fn refresh_token(&self, refresh: &Secret) -> Result<RefreshedToken, ClientError> {
        let res = self
            .request(&pb::ProtoOaRefreshTokenReq {
                payload_type: None,
                refresh_token: refresh.expose().to_string(),
            })
            .await?;
        Ok(RefreshedToken {
            access_token: Secret::new(res.access_token),
            refresh_token: Secret::new(res.refresh_token),
            token_type: res.token_type,
            expires_in_secs: res.expires_in,
        })
    }

    /// Which accounts the access token's grant covers
    /// (`ProtoOAGetAccountListByAccessTokenReq`). Read-only; needs only the
    /// application to be authorised. A refresh renews a grant, it never adds
    /// an account to it.
    pub async fn account_list(&self, access_token: &Secret) -> Result<TokenGrant, ClientError> {
        let res = self
            .request(&pb::ProtoOaGetAccountListByAccessTokenReq {
                payload_type: None,
                access_token: access_token.expose().to_string(),
            })
            .await?;
        Ok(TokenGrant {
            permission_scope: res.permission_scope.map(|s| {
                pb::ProtoOaClientPermissionScope::try_from(s)
                    .map(|v| v.as_str_name().to_string())
                    .unwrap_or_else(|_| format!("UNKNOWN({s})"))
            }),
            accounts: res
                .ctid_trader_account
                .iter()
                .map(|a| GrantedAccount {
                    ctid: a.ctid_trader_account_id as i64,
                    is_live: a.is_live,
                    trader_login: a.trader_login,
                })
                .collect(),
        })
    }

    /// Authorise one account. Only a [`PermittedAccount`] — an account that
    /// passed the allowlist — can be authorised.
    pub async fn account_auth(
        &self,
        account: &PermittedAccount,
        access_token: &Secret,
    ) -> Result<AccountSession<'_>, ClientError> {
        let res = self
            .request(&pb::ProtoOaAccountAuthReq {
                payload_type: None,
                ctid_trader_account_id: account.id(),
                access_token: access_token.expose().to_string(),
            })
            .await?;
        if res.ctid_trader_account_id != account.id() {
            return Err(ClientError::WrongAccount {
                asked: account.id(),
                got: res.ctid_trader_account_id,
            });
        }
        Ok(AccountSession {
            conn: self,
            account: account.clone(),
        })
    }

    async fn historical_pause(&self) {
        let mut last = self.historical_gate.lock().await;
        if let Some(t) = *last {
            let since = t.elapsed();
            if since < HISTORICAL_MIN_GAP {
                tokio::time::sleep(HISTORICAL_MIN_GAP - since).await;
            }
        }
        *last = Some(Instant::now());
    }
}

async fn send_checked(
    writer: &Mutex<Writer>,
    payload_type: u32,
    payload: Vec<u8>,
    client_msg_id: Option<String>,
) -> Result<(), ClientError> {
    // The allowlist gate. Nothing reaches the socket without passing it.
    check_sendable(payload_type)?;
    let msg = ProtoMessage {
        payload_type,
        payload: Some(payload),
        client_msg_id,
    };
    let mut w = writer.lock().await;
    write_frame(&mut *w, &msg).await?;
    Ok(())
}

fn route(
    msg: ProtoMessage,
    pending: &Pending,
    spots: &broadcast::Sender<ReceivedSpot>,
    notices: &StdMutex<Vec<String>>,
    ends: &(
        Arc<StdMutex<Vec<SessionEnd>>>,
        broadcast::Sender<SessionEnd>,
    ),
) {
    let end = |account_ids: Vec<i64>, reason: String| {
        let e = SessionEnd {
            account_ids,
            reason,
        };
        ends.0.lock().unwrap().push(e.clone());
        let _ = ends.1.send(e);
    };
    if let Some(id) = &msg.client_msg_id {
        if let Some(tx) = pending.lock().unwrap().remove(id) {
            let _ = tx.send(msg);
            return;
        }
    }
    let payload = msg.payload.as_deref().unwrap_or(&[]);
    let pt = msg.payload_type as i32;
    if pt == Oa::ProtoOaSpotEvent as i32 {
        if let Ok(event) = ProtoOaSpotEvent::decode(payload) {
            let _ = spots.send(ReceivedSpot {
                received_at: Utc::now(),
                event,
            });
        }
    } else if pt == Common::HeartbeatEvent as i32 {
        // The proxy's keep-alive. Nothing to do.
    } else if pt == Oa::ProtoOaAccountsTokenInvalidatedEvent as i32 {
        let ev = pb::ProtoOaAccountsTokenInvalidatedEvent::decode(payload).unwrap_or_default();
        let note = format!(
            "accounts token invalidated for {:?}: {}",
            ev.ctid_trader_account_ids,
            ev.reason.unwrap_or_default()
        );
        tracing::warn!("{note}");
        notices.lock().unwrap().push(note.clone());
        // An event that names no account is taken to end every session on
        // this connection (`SessionEnd` with an empty list).
        end(ev.ctid_trader_account_ids, note);
    } else if pt == Oa::ProtoOaClientDisconnectEvent as i32 {
        let ev = pb::ProtoOaClientDisconnectEvent::decode(payload).unwrap_or_default();
        let note = format!("client disconnected: {}", ev.reason.unwrap_or_default());
        tracing::warn!("{note}");
        notices.lock().unwrap().push(note.clone());
        end(vec![], note);
    } else if pt == Oa::ProtoOaAccountDisconnectEvent as i32 {
        let ev = pb::ProtoOaAccountDisconnectEvent::decode(payload).unwrap_or_default();
        let note = format!("account {} disconnected", ev.ctid_trader_account_id);
        tracing::warn!("{note}");
        notices.lock().unwrap().push(note.clone());
        end(vec![ev.ctid_trader_account_id], note);
    } else {
        tracing::debug!(
            payload_type = msg.payload_type,
            name = payload_name(msg.payload_type),
            "unsolicited message ignored"
        );
    }
}

fn decode_response<T: Response>(msg: ProtoMessage) -> Result<T, ClientError> {
    let payload = msg.payload.as_deref().unwrap_or(&[]);
    if msg.payload_type == T::PAYLOAD_TYPE {
        return Ok(T::decode(payload)?);
    }
    if msg.payload_type == Oa::ProtoOaErrorRes as u32 {
        let e = ProtoOaErrorRes::decode(payload)?;
        return Err(ClientError::Broker {
            code: e.error_code,
            description: e.description,
            retry_after_secs: e.retry_after,
        });
    }
    if msg.payload_type == Common::ErrorRes as u32 {
        let e = ProtoErrorRes::decode(payload)?;
        return Err(ClientError::Broker {
            code: e.error_code,
            description: e.description,
            retry_after_secs: None,
        });
    }
    Err(ClientError::UnexpectedPayload {
        expected: T::PAYLOAD_TYPE,
        got: msg.payload_type,
    })
}

/// An authorised account on a connection.
pub struct AccountSession<'c> {
    conn: &'c Connection,
    account: PermittedAccount,
}

/// Spread samples for one symbol, in relative units (1/100000 of a price).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpreadSamples {
    /// ask − bid at each quote update where both sides were known.
    pub spreads: Vec<i64>,
}

impl SpreadSamples {
    /// The median spread in price units, exact. `None` without samples.
    pub fn median(&self) -> Option<Decimal> {
        let mut v = self.spreads.clone();
        if v.is_empty() {
            return None;
        }
        v.sort_unstable();
        let n = v.len();
        Some(if n % 2 == 1 {
            Decimal::new(v[n / 2], 5)
        } else {
            // (a + b) / 2 / 100000 = (a + b) × 5 / 10^6, exact.
            Decimal::new((v[n / 2 - 1] + v[n / 2]) * 5, 6).normalize()
        })
    }
}

impl AccountSession<'_> {
    pub fn account(&self) -> &PermittedAccount {
        &self.account
    }

    /// The broker ended this account's session on its connection (token
    /// invalidated, account or client disconnected); it must re-authorise.
    pub fn session_ended(&self) -> Option<SessionEnd> {
        self.conn.session_ended(self.account.id())
    }

    /// Why the connection under this session closed, once it has.
    pub fn closed_reason(&self) -> Option<String> {
        self.conn.closed_reason()
    }

    fn check_account(&self, got: i64) -> Result<(), ClientError> {
        if got == self.account.id() {
            Ok(())
        } else {
            Err(ClientError::WrongAccount {
                asked: self.account.id(),
                got,
            })
        }
    }

    /// The account's assets (currencies and the like): id → name, as the
    /// broker states them (`ProtoOAAssetListReq`, read-only).
    pub async fn asset_list(&self) -> Result<Vec<Asset>, ClientError> {
        let res = self
            .conn
            .request(&pb::ProtoOaAssetListReq {
                payload_type: None,
                ctid_trader_account_id: self.account.id(),
            })
            .await?;
        self.check_account(res.ctid_trader_account_id)?;
        Ok(res
            .asset
            .into_iter()
            .map(|a| Asset {
                asset_id: a.asset_id,
                name: a.name,
                display_name: a.display_name,
                digits: a.digits,
            })
            .collect())
    }

    pub async fn symbols_list(&self) -> Result<pb::ProtoOaSymbolsListRes, ClientError> {
        let res = self
            .conn
            .request(&pb::ProtoOaSymbolsListReq {
                payload_type: None,
                ctid_trader_account_id: self.account.id(),
                include_archived_symbols: Some(false),
            })
            .await?;
        self.check_account(res.ctid_trader_account_id)?;
        Ok(res)
    }

    pub async fn symbols_by_id(
        &self,
        symbol_ids: &[i64],
    ) -> Result<pb::ProtoOaSymbolByIdRes, ClientError> {
        let res = self
            .conn
            .request(&pb::ProtoOaSymbolByIdReq {
                payload_type: None,
                ctid_trader_account_id: self.account.id(),
                symbol_id: symbol_ids.to_vec(),
            })
            .await?;
        self.check_account(res.ctid_trader_account_id)?;
        Ok(res)
    }

    /// One D1 trendbar request for `[from, to]`. Returns the bars and `hasMore`.
    /// Waits out a rate-limit refusal (`BLOCKED_PAYLOAD_TYPE` with `retryAfter`)
    /// up to three times.
    pub async fn daily_trendbars(
        &self,
        symbol_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<(Vec<ProtoOaTrendbar>, bool), ClientError> {
        let req = pb::ProtoOaGetTrendbarsReq {
            payload_type: None,
            ctid_trader_account_id: self.account.id(),
            from_timestamp: Some(from.timestamp_millis().max(0)),
            to_timestamp: Some(to.timestamp_millis()),
            period: ProtoOaTrendbarPeriod::D1 as i32,
            symbol_id,
            count: None,
        };
        let mut attempt = 0;
        loop {
            self.conn.historical_pause().await;
            match self.conn.request(&req).await {
                Ok(res) => {
                    self.check_account(res.ctid_trader_account_id)?;
                    if let Some(s) = res.symbol_id {
                        if s != symbol_id {
                            return Err(ClientError::Broker {
                                code: "WRONG_SYMBOL".into(),
                                description: Some(format!("asked {symbol_id}, got {s}")),
                                retry_after_secs: None,
                            });
                        }
                    }
                    return Ok((res.trendbar, res.has_more.unwrap_or(false)));
                }
                Err(ClientError::Broker {
                    code,
                    retry_after_secs: Some(secs),
                    ..
                }) if code == "BLOCKED_PAYLOAD_TYPE" && attempt < 3 => {
                    attempt += 1;
                    tokio::time::sleep(Duration::from_secs(secs.min(60))).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// All daily bars the broker holds for a symbol, newest window first,
    /// walking back one year at a time until [`EMPTY_WINDOWS_TO_STOP`] empty
    /// windows in a row or 1970. A window that reports `hasMore` is split in
    /// half until it does not. Bars are returned raw, de-duplicated by open time.
    pub async fn daily_history(
        &self,
        symbol_id: i64,
        now: DateTime<Utc>,
    ) -> Result<Vec<ProtoOaTrendbar>, ClientError> {
        let epoch = DateTime::<Utc>::UNIX_EPOCH;
        let mut out: HashMap<u32, ProtoOaTrendbar> = HashMap::new();
        let mut to = now;
        let mut empty = 0;
        while to > epoch {
            let from = (to - chrono::Duration::days(D1_WINDOW_DAYS)).max(epoch);
            let bars = self.window(symbol_id, from, to, 0).await?;
            if bars.is_empty() {
                empty += 1;
                if empty >= EMPTY_WINDOWS_TO_STOP {
                    break;
                }
            } else {
                empty = 0;
            }
            for b in bars {
                if let Some(m) = b.utc_timestamp_in_minutes {
                    out.insert(m, b);
                } else {
                    // Kept under a key that cannot collide, so decoding refuses
                    // and records it rather than this function dropping it.
                    out.insert(u32::MAX - out.len() as u32, b);
                }
            }
            to = from - chrono::Duration::milliseconds(1);
        }
        let mut v: Vec<_> = out.into_values().collect();
        v.sort_by_key(|b| b.utc_timestamp_in_minutes);
        Ok(v)
    }

    fn window<'a>(
        &'a self,
        symbol_id: i64,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        depth: u32,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ProtoOaTrendbar>, ClientError>> + Send + 'a,
        >,
    > {
        Box::pin(async move {
            let (bars, has_more) = self.daily_trendbars(symbol_id, from, to).await?;
            if !has_more || depth >= 12 || to - from <= chrono::Duration::days(2) {
                return Ok(bars);
            }
            let mid = from + (to - from) / 2;
            let mut a = self.window(symbol_id, from, mid, depth + 1).await?;
            let b = self
                .window(
                    symbol_id,
                    mid + chrono::Duration::milliseconds(1),
                    to,
                    depth + 1,
                )
                .await?;
            a.extend(b);
            Ok(a)
        })
    }

    /// A historical-data request: spaced by [`HISTORICAL_MIN_GAP`], and a
    /// rate-limit refusal (`BLOCKED_PAYLOAD_TYPE` with `retryAfter`) is waited
    /// out up to three times.
    async fn historical<R: Request>(&self, req: &R) -> Result<R::Response, ClientError> {
        let mut attempt = 0;
        loop {
            self.conn.historical_pause().await;
            match self.conn.request(req).await {
                Err(ClientError::Broker {
                    code,
                    retry_after_secs: Some(secs),
                    ..
                }) if code == "BLOCKED_PAYLOAD_TYPE" && attempt < 3 => {
                    attempt += 1;
                    tokio::time::sleep(Duration::from_secs(secs.min(60))).await;
                }
                other => return other,
            }
        }
    }

    /// The account's trader record (balance, deposit asset, registration time).
    pub async fn trader(&self) -> Result<pb::ProtoOaTrader, ClientError> {
        let res = self
            .conn
            .request(&pb::ProtoOaTraderReq {
                payload_type: None,
                ctid_trader_account_id: self.account.id(),
            })
            .await?;
        self.check_account(res.ctid_trader_account_id)?;
        Ok(res.trader)
    }

    /// Every bid or ask tick of `[from_ms, to_ms]` (at most one week), oldest
    /// first. Follows `hasMore` page by page (see [`crate::tickdata::Pages`]).
    pub async fn tick_history(
        &self,
        symbol_id: i64,
        side: crate::tickdata::QuoteSide,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<crate::tickdata::TickFetch, ClientError> {
        use crate::tickdata::{decode_ticks, Pages, TickDataError, MAX_WINDOW_MS};
        if to_ms - from_ms > MAX_WINDOW_MS || to_ms < from_ms {
            return Err(TickDataError::WindowTooLong { from_ms, to_ms }.into());
        }
        let mut pages = Pages::default();
        loop {
            let to = pages.next_to().unwrap_or(to_ms);
            let res = self
                .historical(&pb::ProtoOaGetTickDataReq {
                    payload_type: None,
                    ctid_trader_account_id: self.account.id(),
                    symbol_id,
                    r#type: side.proto(),
                    from_timestamp: Some(from_ms),
                    to_timestamp: Some(to),
                })
                .await?;
            self.check_account(res.ctid_trader_account_id)?;
            let (ticks, sign) = decode_ticks(&res.tick_data, from_ms, to)?;
            let empty = ticks.is_empty();
            pages.push(ticks, sign)?;
            if !res.has_more || empty {
                break;
            }
        }
        Ok(pages.into_fetch())
    }

    /// One page of ticks only (for probing whether a window has any): the
    /// decoded ticks of the first response and its `hasMore`.
    pub async fn tick_page(
        &self,
        symbol_id: i64,
        side: crate::tickdata::QuoteSide,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<(Vec<crate::tickdata::HistTick>, bool), ClientError> {
        use crate::tickdata::{decode_ticks, TickDataError, MAX_WINDOW_MS};
        if to_ms - from_ms > MAX_WINDOW_MS || to_ms < from_ms {
            return Err(TickDataError::WindowTooLong { from_ms, to_ms }.into());
        }
        let res = self
            .historical(&pb::ProtoOaGetTickDataReq {
                payload_type: None,
                ctid_trader_account_id: self.account.id(),
                symbol_id,
                r#type: side.proto(),
                from_timestamp: Some(from_ms),
                to_timestamp: Some(to_ms),
            })
            .await?;
        self.check_account(res.ctid_trader_account_id)?;
        let (ticks, _) = decode_ticks(&res.tick_data, from_ms, to_ms)?;
        Ok((ticks, res.has_more))
    }

    /// Every deal of `[from_ms, to_ms]`. A window that reports `hasMore` is
    /// split in half until it does not.
    pub async fn deal_history(
        &self,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<pb::ProtoOaDeal>, ClientError> {
        let mut out = Vec::new();
        let mut stack = vec![(from_ms, to_ms, 0u32)];
        while let Some((from, to, depth)) = stack.pop() {
            let res = self
                .historical(&pb::ProtoOaDealListReq {
                    payload_type: None,
                    ctid_trader_account_id: self.account.id(),
                    from_timestamp: Some(from),
                    to_timestamp: Some(to),
                    max_rows: None,
                })
                .await?;
            self.check_account(res.ctid_trader_account_id)?;
            if res.has_more && to - from > 1 && depth < 30 {
                let mid = from + (to - from) / 2;
                stack.push((mid + 1, to, depth + 1));
                stack.push((from, mid, depth + 1));
                continue;
            }
            if res.has_more {
                return Err(ClientError::Broker {
                    code: "DEALS_TRUNCATED".into(),
                    description: Some(format!(
                        "more deals in {from}..{to} than one response holds"
                    )),
                    retry_after_secs: None,
                });
            }
            out.extend(res.deal);
        }
        out.sort_by_key(|d| (d.execution_timestamp, d.deal_id));
        out.dedup_by_key(|d| d.deal_id);
        Ok(out)
    }

    pub async fn subscribe_spots(&self, symbol_ids: &[i64]) -> Result<(), ClientError> {
        let res = self
            .conn
            .request(&pb::ProtoOaSubscribeSpotsReq {
                payload_type: None,
                ctid_trader_account_id: self.account.id(),
                symbol_id: symbol_ids.to_vec(),
                subscribe_to_spot_timestamp: Some(true),
            })
            .await?;
        self.check_account(res.ctid_trader_account_id)
    }

    pub async fn unsubscribe_spots(&self, symbol_ids: &[i64]) -> Result<(), ClientError> {
        let res = self
            .conn
            .request(&pb::ProtoOaUnsubscribeSpotsReq {
                payload_type: None,
                ctid_trader_account_id: self.account.id(),
                symbol_id: symbol_ids.to_vec(),
            })
            .await?;
        self.check_account(res.ctid_trader_account_id)
    }

    /// Subscribe to spots for all `symbol_ids` at once, record ask − bid at every
    /// update for `window`, then unsubscribe. A spot event may carry only the
    /// side that changed, so the last known bid and ask are carried forward;
    /// no sample is taken until both sides are known.
    pub async fn sample_spreads(
        &self,
        symbol_ids: &[i64],
        window: Duration,
    ) -> Result<HashMap<i64, SpreadSamples>, ClientError> {
        let mut rx = self.conn.spot_events();
        let mut ends = self.conn.session_ends();
        self.subscribe_spots(symbol_ids).await?;
        if let Some(e) = self.conn.session_ended(self.account.id()) {
            return Err(ClientError::Disconnected(e.reason));
        }
        let mut last: HashMap<i64, (Option<u64>, Option<u64>)> = HashMap::new();
        let mut out: HashMap<i64, SpreadSamples> = symbol_ids
            .iter()
            .map(|id| (*id, SpreadSamples::default()))
            .collect();
        let deadline = Instant::now() + window;
        loop {
            let next = tokio::select! {
                r = rx.recv() => r,
                e = ends.recv() => match e {
                    Ok(e) if e.affects(self.account.id()) => {
                        return Err(ClientError::Disconnected(e.reason));
                    }
                    _ => continue,
                },
                _ = tokio::time::sleep_until(deadline) => break,
            };
            let ev = match next {
                Ok(r) => r.event,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(missed = n, "spot sampling lagged");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            };
            if ev.ctid_trader_account_id != self.account.id() {
                continue;
            }
            let Some(samples) = out.get_mut(&ev.symbol_id) else {
                continue;
            };
            let side = last.entry(ev.symbol_id).or_default();
            if ev.bid.is_some() {
                side.0 = ev.bid;
            }
            if ev.ask.is_some() {
                side.1 = ev.ask;
            }
            if let (Some(bid), Some(ask)) = *side {
                if let (Ok(bid), Ok(ask)) = (i64::try_from(bid), i64::try_from(ask)) {
                    samples.spreads.push(ask - bid);
                }
            }
        }
        // Unsubscribe even if the window ended on a closed connection; report
        // the unsubscribe failure only if sampling itself succeeded.
        self.unsubscribe_spots(symbol_ids).await?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_odd_even_and_empty() {
        let s = |v: Vec<i64>| SpreadSamples { spreads: v };
        assert_eq!(s(vec![]).median(), None);
        assert_eq!(s(vec![30, 10, 20]).median(), Some(Decimal::new(20, 5)));
        // (10 + 15) / 2 = 12.5 relative = 0.000125
        assert_eq!(s(vec![15, 10]).median(), Some(Decimal::new(125, 6)));
    }

    #[tokio::test]
    async fn a_hand_built_trading_frame_is_refused_before_any_byte_is_written() {
        let (a, mut b) = tokio::io::duplex(1024);
        let conn = Connection::from_stream(a, Duration::from_secs(1));
        let err = conn
            .send_message(
                Oa::ProtoOaNewOrderReq as u32,
                vec![1, 2, 3],
                Some("x".into()),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, ClientError::NotSendable(_)), "{err}");
        drop(conn);
        // Nothing reached the other end.
        let mut buf = Vec::new();
        use tokio::io::AsyncReadExt;
        let n = b.read_to_end(&mut buf).await.unwrap();
        assert_eq!(n, 0, "bytes written: {buf:?}");
    }
}
