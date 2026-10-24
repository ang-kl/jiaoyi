//! The phase-1 load of one permitted account.
//!
//! Every tunable number is read from the settings at the start of the load,
//! and the settings version is stored on the load run.
//!
//! Order: authorise → list symbols → resolve the universe (recorded) → read and
//! store symbol details → read the maximum daily history and store closed bars
//! → sample spreads and store costs → cross-check recent bars against a fresh
//! broker read → close the load run.
//!
//! Nothing here can send an order: the client has no trading request on its
//! payload allowlist.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use chrono::{DateTime, Utc};
use jiaoyi_core::allowlist::{Environment, PermittedAccount};
use jiaoyi_core::secret::Secret;
use jiaoyi_core::universe::{resolve_universe, AssetClass, ListedSymbol};
use jiaoyi_ctrader::client::SpreadSamples;
use jiaoyi_ctrader::messages::{
    ProtoOaCommissionType, ProtoOaDayOfWeek, ProtoOaMinCommissionType, ProtoOaSwapCalculationType,
    ProtoOaSymbol, ProtoOaTradingMode,
};
use jiaoyi_ctrader::{
    closed_daily_bars, decode_trendbar, AccountSession, ClientError, Connection, TokenGrant,
};
use jiaoyi_store::sqlx::PgPool;
use jiaoyi_store::{self as store, BarRow, CostRow, RunSummary, SymbolRow};
use prost::Message;
use serde_json::{json, Map, Value};

use crate::config::BrokerConfig;

#[derive(Debug, Clone, PartialEq)]
pub struct LoadOutcome {
    pub run_id: i64,
    pub status: &'static str,
    pub summary: RunSummary,
    /// The run failed because the broker rejected the access token.
    pub auth_rejected: bool,
}

/// Get the token to use. `None` means there is no token anywhere.
///
/// The stored token, unless the environment holds a NEW seed (PR #6): a seed
/// whose fingerprint differs from the one the stored token descends from
/// replaces it, and the replacement is recorded (fingerprints only). The same
/// seed keeps the stored, refreshed token. A token stored before seed
/// fingerprints were kept is judged once: see [`judge_legacy_seed`].
/// `conn` must have the application authorised (the legacy case reads grants).
///
/// With a seed in the environment, the whole decision (read the stored row,
/// compare fingerprints, judge a legacy row, store the seed) is one critical
/// section under [`REFRESH_LOCK`], the lock [`refresh`] takes. On boot the
/// loader, the recorders and the history syncs all get here at once; without
/// it a task holding a stale read could write the seed back over the token
/// another task had already refreshed from it, leaving a spent refresh token
/// and an invalidated access token (Codex P1 on #6). A task that waited
/// re-reads the row, finds the seed's fingerprint, and keeps what is stored.
pub async fn current_token(
    pool: &PgPool,
    conn: &Connection,
    broker: &BrokerConfig,
) -> anyhow::Result<Option<store::StoredToken>> {
    let (Some(a), Some(r)) = (&broker.seed_access_token, &broker.seed_refresh_token) else {
        return Ok(store::load_token(pool).await?);
    };
    let seed = a.fingerprint();
    let _decision = REFRESH_LOCK.lock().await;
    let stored = store::load_token(pool).await?;
    let Some(t) = stored else {
        store::store_seed(pool, a, r, broker.seed_expires_at).await?;
        return Ok(store::load_token(pool).await?);
    };
    match t.seed_fingerprint.clone() {
        Some(fp) if fp == seed => return Ok(Some(t)),
        Some(old) => {
            replace_with_seed(
                pool,
                broker,
                &t,
                Some(&old),
                "the environment holds a new seed",
            )
            .await?
        }
        None => {
            if let Some(why) = judge_legacy_seed(pool, conn, &t, a).await? {
                replace_with_seed(pool, broker, &t, None, &why).await?;
            }
        }
    }
    Ok(store::load_token(pool).await?)
}

/// A token stored before migration 0007 has no seed fingerprint, and on the
/// deploy it had been refreshed: storing the env seed over it blindly could
/// put back a spent token. Returns why the seed should replace it, or `None`
/// to keep it (the seed's fingerprint is then adopted as the baseline).
///
/// - stored from the env seed: replace when the seed differs from it;
/// - stored from a refresh: read the seed's grant. Refused: the seed is
///   spent, keep the stored token. Accepted: replace when the stored token is
///   refused, or when the seed's grant lists an account the stored token's
///   grant does not (a new grant); otherwise keep.
async fn judge_legacy_seed(
    pool: &PgPool,
    conn: &Connection,
    t: &store::StoredToken,
    seed_access: &Secret,
) -> anyhow::Result<Option<String>> {
    let seed = seed_access.fingerprint();
    let keep = |detail: Value| async move {
        store::set_seed_fingerprint(pool, &seed_access.fingerprint()).await?;
        store::record_reconcile(
            pool,
            None,
            "token_seed_baseline",
            "oauth_tokens",
            detail,
            "the stored token is kept; the env seed's fingerprint is adopted as its baseline",
        )
        .await?;
        anyhow::Ok(None)
    };
    if t.source == "env_seed" {
        if t.access_token.fingerprint() == seed {
            return keep(
                json!({ "seed_fingerprint": seed, "why": "the stored token is this seed" }),
            )
            .await;
        }
        return Ok(Some(
            "stored before seed fingerprints were kept, and the env seed differs from it".into(),
        ));
    }
    let ids = |g: &TokenGrant| g.accounts.iter().map(|a| a.ctid).collect::<Vec<_>>();
    let seed_grant = match conn.account_list(seed_access).await {
        Ok(g) => g,
        Err(e) if e.is_token_rejected() => {
            return keep(json!({
                "seed_fingerprint": seed,
                "stored_token_fingerprint": t.access_token.fingerprint(),
                "why": format!("the env seed's token is refused ({e}): it is spent"),
            }))
            .await
        }
        Err(e) => return Err(e.into()),
    };
    match conn.account_list(&t.access_token).await {
        Err(e) if e.is_token_rejected() => Ok(Some(format!(
            "the stored token is refused ({e}) and the env seed is accepted"
        ))),
        Err(e) => Err(e.into()),
        Ok(stored_grant) => {
            let added: Vec<i64> = ids(&seed_grant)
                .into_iter()
                .filter(|id| stored_grant.account(*id).is_none())
                .collect();
            if added.is_empty() {
                keep(json!({
                    "seed_fingerprint": seed,
                    "stored_token_fingerprint": t.access_token.fingerprint(),
                    "seed_grant": ids(&seed_grant),
                    "stored_grant": ids(&stored_grant),
                    "why": "the env seed's grant adds no account",
                }))
                .await
            } else {
                Ok(Some(format!(
                    "the env seed's grant adds accounts {added:?} to the stored token's {:?}",
                    ids(&stored_grant)
                )))
            }
        }
    }
}

/// Store the env seed over the stored token and record the replacement, by
/// fingerprints only.
async fn replace_with_seed(
    pool: &PgPool,
    broker: &BrokerConfig,
    t: &store::StoredToken,
    old_seed: Option<&str>,
    why: &str,
) -> anyhow::Result<()> {
    let (Some(a), Some(r)) = (&broker.seed_access_token, &broker.seed_refresh_token) else {
        return Ok(());
    };
    store::store_seed(pool, a, r, broker.seed_expires_at).await?;
    store::record_reconcile(
        pool,
        None,
        "token_seed_replaced",
        "oauth_tokens",
        json!({
            "replaced_token_fingerprint": t.access_token.fingerprint(),
            "replaced_token_source": t.source,
            "replaced_seed_fingerprint": old_seed,
            "new_seed_fingerprint": a.fingerprint(),
            "why": why,
        }),
        "the new env seed is stored and used; the replaced token is no longer used",
    )
    .await?;
    tracing::warn!(
        replaced = %t.access_token.fingerprint(),
        new_seed = %a.fingerprint(),
        why,
        "a new env seed replaces the stored token"
    );
    Ok(())
}

/// Why an account is not authorised, read from the token's grant
/// (`ProtoOAGetAccountListByAccessTokenReq`). Neither is a token rejection: a
/// refresh renews a grant and never adds an account to it, so neither is
/// refreshed for (PR #6; deploy 7765158a).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GrantError {
    #[error("token grant does not include account {account} (grant lists: [{}])", join_ids(.listed))]
    NotListed { account: i64, listed: Vec<i64> },
    #[error(
        "token grant lists account {account} as {}, but it is declared {}; refused",
        if *.is_live { "live" } else { "demo" },
        .declared.as_str()
    )]
    EnvironmentMismatch {
        account: i64,
        declared: Environment,
        is_live: bool,
    },
}

impl GrantError {
    pub fn verdict(&self) -> &'static str {
        match self {
            GrantError::NotListed { .. } => "not_listed",
            GrantError::EnvironmentMismatch { .. } => "environment_mismatch",
        }
    }
}

fn join_ids(ids: &[i64]) -> String {
    ids.iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// True when an error chain holds a [`GrantError`]: the account cannot be
/// authorised with this token, and retrying soon will not change that.
pub fn is_grant_refusal(e: &anyhow::Error) -> bool {
    e.chain().any(|c| c.downcast_ref::<GrantError>().is_some())
}

/// Is `account` in the grant, with an environment that agrees with the one
/// it was declared with? An `isLive` the broker does not send is no
/// disagreement.
pub fn judge_grant(
    account: i64,
    declared: Option<Environment>,
    grant: &TokenGrant,
) -> Result<(), GrantError> {
    let Some(a) = grant.account(account) else {
        return Err(GrantError::NotListed {
            account,
            listed: grant.accounts.iter().map(|a| a.ctid).collect(),
        });
    };
    match (declared, a.is_live) {
        (Some(env), Some(live)) if (env == Environment::Live) != live => {
            Err(GrantError::EnvironmentMismatch {
                account,
                declared: env,
                is_live: live,
            })
        }
        _ => Ok(()),
    }
}

/// The last grant state logged per account: a state is logged when it
/// changes, not on every retry.
static GRANT_STATES: std::sync::Mutex<BTreeMap<i64, String>> =
    std::sync::Mutex::new(BTreeMap::new());

fn state_changed(account: i64, state: String) -> bool {
    let mut m = GRANT_STATES.lock().unwrap_or_else(|e| e.into_inner());
    if m.get(&account) == Some(&state) {
        false
    } else {
        m.insert(account, state);
        true
    }
}

/// Read the token's grant for `account` and record the read. The broker's
/// refusal of the read itself (an expired or invalid token) is recorded and
/// returned as the client error.
async fn read_grant(
    pool: &PgPool,
    conn: &Connection,
    account: i64,
    declared: Option<Environment>,
    host: &str,
    token: &store::StoredToken,
) -> anyhow::Result<Result<Result<(), GrantError>, ClientError>> {
    let mut read = store::GrantRead {
        account_id: account,
        host: host.to_string(),
        token_fingerprint: token.access_token.fingerprint(),
        token_source: token.source.clone(),
        verdict: "listed",
        reason: None,
        permission_scope: None,
        accounts: None,
    };
    let outcome = match conn.account_list(&token.access_token).await {
        Ok(g) => {
            read.permission_scope = g.permission_scope.clone();
            read.accounts = Some(json!(g
                .accounts
                .iter()
                .map(|a| json!({ "ctid": a.ctid, "is_live": a.is_live, "login": a.trader_login }))
                .collect::<Vec<_>>()));
            let judged = judge_grant(account, declared, &g);
            if let Err(e) = &judged {
                read.verdict = e.verdict();
                read.reason = Some(e.to_string());
            }
            Ok(judged)
        }
        Err(e) if e.is_token_rejected() => {
            read.verdict = "rejected";
            read.reason = Some(format!("the account-list request was refused: {e}"));
            Err(e)
        }
        Err(e) => return Err(e.into()),
    };
    store::record_grant(pool, &read).await?;
    if state_changed(
        account,
        format!("{} {}", read.verdict, read.token_fingerprint),
    ) {
        match read.verdict {
            "listed" => tracing::info!(
                account,
                token = %read.token_fingerprint,
                accounts = %read.accounts.as_ref().map(|a| a.to_string()).unwrap_or_default(),
                "token grant includes the account"
            ),
            v => tracing::warn!(
                account,
                token = %read.token_fingerprint,
                state = v,
                reason = %read.reason.clone().unwrap_or_default(),
                "token grant state changed"
            ),
        }
    }
    Ok(outcome)
}

/// One refresh at a time in this process: the refresh token is single-use,
/// and the loader, the tick recorders and the history syncs share it. The
/// env-seed decision in [`current_token`] holds it too.
static REFRESH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Refresh the token over this connection and store the new pair. If another
/// task refreshed it meanwhile, use that token instead of refreshing again.
pub async fn refresh(
    pool: &PgPool,
    conn: &Connection,
    token: &store::StoredToken,
) -> anyhow::Result<store::StoredToken> {
    let _guard = REFRESH_LOCK.lock().await;
    if let Some(now_stored) = store::load_token(pool).await? {
        if now_stored.refresh_token != token.refresh_token {
            return Ok(now_stored);
        }
    }
    let t = conn.refresh_token(&token.refresh_token).await?;
    let expires_at = Utc::now() + chrono::Duration::seconds(t.expires_in_secs);
    store::save_token(
        pool,
        &t.access_token,
        &t.refresh_token,
        Some(&t.token_type),
        Some(expires_at),
        "refresh",
    )
    .await?;
    tracing::info!(expires_at = %expires_at, "cTrader access token refreshed and stored");
    store::load_token(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("token vanished after refresh"))
}

/// Refresh the token when it expires within this window.
pub const REFRESH_WHEN_WITHIN: chrono::Duration = chrono::Duration::days(3);

/// Load one account over an open connection. Records a load run whatever happens.
pub async fn load_account(
    pool: &PgPool,
    conn: &Connection,
    broker: &BrokerConfig,
    account: &PermittedAccount,
) -> store::Result<LoadOutcome> {
    let run_id = store::start_run(pool, account.id()).await?;
    let mut summary = RunSummary::default();
    let mut auth_rejected = false;
    let status = match run(pool, conn, broker, account, run_id, &mut summary).await {
        Ok(()) if summary.errors.is_empty() => "ok",
        Ok(()) => "partial",
        Err(e) => {
            auth_rejected = e.chain().any(|c| {
                c.downcast_ref::<ClientError>()
                    .is_some_and(ClientError::is_token_rejected)
            });
            summary.errors.push(format!("{e:#}"));
            "failed"
        }
    };
    for n in conn.notices() {
        summary.errors.push(format!("broker notice: {n}"));
    }
    store::finish_run(pool, run_id, status, &summary).await?;
    tracing::info!(account = %account, run_id, status, ?summary.errors, "load finished");
    Ok(LoadOutcome {
        run_id,
        status,
        summary,
        auth_rejected,
    })
}

async fn run(
    pool: &PgPool,
    conn: &Connection,
    broker: &BrokerConfig,
    account: &PermittedAccount,
    run_id: i64,
    summary: &mut RunSummary,
) -> anyhow::Result<()> {
    let settings = store::settings::snapshot(pool, Some(account.id())).await?;
    summary.settings_version = Some(settings.version);
    let universe = settings
        .universe("universe.symbols")
        .map_err(anyhow::Error::msg)?;
    let spread_window = Duration::from_secs(
        settings
            .integer("spread.sample_secs")
            .map_err(anyhow::Error::msg)?
            .max(0) as u64,
    );
    let crosscheck_per_class = settings
        .integer("crosscheck.symbols_per_class")
        .map_err(anyhow::Error::msg)?
        .max(0) as usize;
    let crosscheck_days = settings
        .integer("crosscheck.days")
        .map_err(anyhow::Error::msg)?;

    let session = authorise(pool, conn, broker, account).await?;

    let listed = session.symbols_list().await?;
    let listed_symbols: Vec<ListedSymbol> = listed
        .symbol
        .iter()
        .map(|s| ListedSymbol {
            symbol_id: s.symbol_id,
            name: s.symbol_name.clone(),
            enabled: s.enabled,
        })
        .collect();
    // The broker's base and quote asset ids are kept too: with the trader
    // record's deposit asset they establish the deposit currency (R1).
    summary.listed_symbols = Some(serde_json::Value::Array(
        listed
            .symbol
            .iter()
            .map(|s| {
                json!({
                    "id": s.symbol_id, "name": s.symbol_name, "enabled": s.enabled,
                    "base_asset_id": s.base_asset_id, "quote_asset_id": s.quote_asset_id,
                })
            })
            .collect(),
    ));
    let resolutions = resolve_universe(&universe, &listed_symbols);
    let mut resolved: Vec<(&str, AssetClass, i64)> = Vec::new();
    for (name, class, r) in &resolutions {
        store::record_resolution(pool, run_id, account.id(), name, *class, r).await?;
        match r.symbol_id() {
            Some(id) => resolved.push((name.as_str(), *class, id)),
            None => summary.symbols_unresolved += 1,
        }
    }
    summary.symbols_resolved = resolved.len() as i32;
    if resolved.is_empty() {
        return Ok(());
    }

    let ids: Vec<i64> = resolved.iter().map(|r| r.2).collect();
    let details = session.symbols_by_id(&ids).await?;
    let by_id: HashMap<i64, ProtoOaSymbol> = details
        .symbol
        .into_iter()
        .map(|s| (s.symbol_id, s))
        .collect();
    let read_at = Utc::now();
    let mut stored: Vec<(&str, AssetClass, i64)> = Vec::new();
    for (name, class, id) in &resolved {
        let Some(sym) = by_id.get(id) else {
            summary.errors.push(format!(
                "{name}: symbol {id} missing from ProtoOASymbolByIdRes; not loaded"
            ));
            continue;
        };
        let enabled = listed_symbols
            .iter()
            .find(|s| s.symbol_id == *id)
            .and_then(|s| s.enabled);
        store::upsert_symbol(
            pool,
            &symbol_row(account.id(), name, *class, enabled, sym, read_at),
        )
        .await?;
        stored.push((name, *class, *id));
    }

    for (name, _, id) in &stored {
        if let Err(e) = load_bars(pool, &session, account.id(), *id, run_id, summary).await {
            summary.errors.push(format!("{name}: daily bars: {e}"));
        }
    }

    let stored_ids: Vec<i64> = stored.iter().map(|s| s.2).collect();
    let samples = match session.sample_spreads(&stored_ids, spread_window).await {
        Ok(s) => s,
        Err(e) => {
            summary.errors.push(format!("spread sampling: {e}"));
            HashMap::new()
        }
    };
    let measured_at = Utc::now();
    for (name, _, id) in &stored {
        let sample = samples.get(id);
        let row = cost_row(
            account.id(),
            *id,
            run_id,
            measured_at,
            spread_window,
            sample,
            by_id.get(id),
        );
        if row.spread_median.is_some() {
            summary.spreads_measured += 1;
        }
        if let Err(e) = store::insert_cost(pool, &row).await {
            summary.errors.push(format!("{name}: costs: {e}"));
        }
    }

    let mut per_class: HashMap<AssetClass, usize> = HashMap::new();
    for (name, class, id) in &stored {
        let n = per_class.entry(*class).or_default();
        if *n >= crosscheck_per_class {
            continue;
        }
        *n += 1;
        match crosscheck(pool, &session, account.id(), *id, crosscheck_days).await {
            Ok(mut v) => {
                v["name"] = json!(name);
                summary.crosscheck.push(v);
            }
            Err(e) => summary.errors.push(format!("{name}: cross-check: {e}")),
        }
    }
    Ok(())
}

/// Authorise the application and one permitted account on a connection.
///
/// After the application is authorised and before the account is, the
/// token's grant is read (PR #6). An account the grant omits, or lists under
/// the other environment, is refused with a [`GrantError`] and the token is
/// NOT refreshed: a refresh would not add it, and it would end every other
/// session using the token. If the grant read itself is refused as an
/// expired or invalid token, the token is refreshed once and the grant read
/// again. A listed account is then authorised, refreshing the token when it is
/// near expiry or rejected (once), as before.
pub async fn authorise<'c>(
    pool: &PgPool,
    conn: &'c Connection,
    broker: &BrokerConfig,
    account: &PermittedAccount,
) -> anyhow::Result<AccountSession<'c>> {
    let client_id = broker
        .client_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CTRADER_CLIENT_ID is not set"))?;
    let client_secret = broker
        .client_secret
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("CTRADER_CLIENT_SECRET is not set"))?;
    conn.application_auth(client_id, client_secret).await?;

    let mut token = current_token(pool, conn, broker).await?.ok_or_else(|| {
        anyhow::anyhow!("no token: oauth_tokens is empty and CTRADER_ACCESS_TOKEN / CTRADER_REFRESH_TOKEN are not set")
    })?;

    // The grant, read before anything here can refresh.
    let declared = store::account_environment(pool, account.id()).await?;
    let host = declared.map_or("unknown", |e| broker.host(e)).to_string();
    // At most one refresh per authorisation: each refresh ends every other
    // session using the old token.
    let mut refreshed = false;
    let judged = match read_grant(pool, conn, account.id(), declared, &host, &token).await? {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!(account = %account, error = %e, "the token's grant read was refused; refreshing once");
            token = refresh(pool, conn, &token).await?;
            refreshed = true;
            read_grant(pool, conn, account.id(), declared, &host, &token).await??
        }
    };
    judged?;

    if !refreshed
        && token
            .expires_at
            .is_some_and(|t| t - Utc::now() < REFRESH_WHEN_WITHIN)
    {
        token = refresh(pool, conn, &token).await?;
        refreshed = true;
    }
    let session = match conn.account_auth(account, &token.access_token).await {
        Ok(s) => s,
        Err(e) if e.is_token_rejected() && !refreshed => {
            tracing::warn!(account = %account, "access token rejected; refreshing once");
            token = refresh(pool, conn, &token).await?;
            conn.account_auth(account, &token.access_token).await?
        }
        Err(e) => return Err(e.into()),
    };

    Ok(session)
}

async fn load_bars(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol_id: i64,
    run_id: i64,
    summary: &mut RunSummary,
) -> anyhow::Result<()> {
    let now = Utc::now();
    let raw = session.daily_history(symbol_id, now).await?;
    let mut decoded = Vec::with_capacity(raw.len());
    for tb in &raw {
        match decode_trendbar(tb) {
            Ok(b) => decoded.push(b),
            Err(e) => {
                summary.bars_refused += 1;
                store::record_reconcile(
                    pool,
                    Some(account_id),
                    "trendbar_refused",
                    &format!("symbol {symbol_id}"),
                    json!({ "error": e.to_string(), "raw_hex": hex(&tb.encode_to_vec()) }),
                    "not stored",
                )
                .await?;
            }
        }
    }
    let (closed, forming) = closed_daily_bars(decoded, now);
    summary.forming_bars_dropped += forming.len() as i32;
    let rows: Vec<BarRow> = closed.into_iter().map(to_row).collect();
    let u = store::upsert_daily_bars(pool, account_id, symbol_id, Some(run_id), &rows, now).await?;
    summary.add_bars(&u);
    Ok(())
}

fn to_row(b: jiaoyi_ctrader::DailyBar) -> BarRow {
    BarRow {
        bar_date: b.date,
        bar_open_utc: b.open_time,
        open: b.open,
        high: b.high,
        low: b.low,
        close: b.close,
        tick_volume: b.tick_volume,
    }
}

/// Re-read the last `days` days from the broker in a separate request and
/// compare with what is stored. Every difference is recorded.
async fn crosscheck(
    pool: &PgPool,
    session: &AccountSession<'_>,
    account_id: i64,
    symbol_id: i64,
    days: i64,
) -> anyhow::Result<Value> {
    let now = Utc::now();
    let (raw, _) = session
        .daily_trendbars(symbol_id, now - chrono::Duration::days(days), now)
        .await?;
    let decoded: Vec<_> = raw.iter().filter_map(|t| decode_trendbar(t).ok()).collect();
    let (closed, _) = closed_daily_bars(decoded, now);
    let stored: HashMap<_, _> = store::bars(pool, account_id, symbol_id)
        .await?
        .into_iter()
        .map(|b| (b.bar_date, b))
        .collect();
    let (mut matched, mut mismatched, mut missing) = (0, 0, 0);
    for b in closed.into_iter().map(to_row) {
        match stored.get(&b.bar_date) {
            Some(s) if *s == b => matched += 1,
            Some(s) => {
                mismatched += 1;
                store::record_reconcile(
                    pool,
                    Some(account_id),
                    "crosscheck_mismatch",
                    &format!("symbol {symbol_id} {}", b.bar_date),
                    json!({ "stored": s, "broker": b }),
                    "recorded; the next load stores the broker value",
                )
                .await?;
            }
            None => {
                missing += 1;
                store::record_reconcile(
                    pool,
                    Some(account_id),
                    "crosscheck_missing",
                    &format!("symbol {symbol_id} {}", b.bar_date),
                    json!({ "broker": b }),
                    "recorded; the next load stores it",
                )
                .await?;
            }
        }
    }
    Ok(json!({
        "symbol_id": symbol_id,
        "window_days": days,
        "matched": matched,
        "mismatched": mismatched,
        "missing_in_db": missing,
    }))
}

fn enum_name<E: TryFrom<i32> + Copy>(
    v: Option<i32>,
    name: impl Fn(E) -> &'static str,
) -> Option<String> {
    v.map(|i| match E::try_from(i) {
        Ok(e) => name(e).to_string(),
        Err(_) => format!("UNKNOWN({i})"),
    })
}

fn note_null<T>(reasons: &mut Map<String, Value>, column: &str, proto_field: &str, v: &Option<T>) {
    if v.is_none() {
        reasons.insert(
            column.into(),
            json!(format!("broker sent no {proto_field}")),
        );
    }
}

pub fn symbol_row(
    account_id: i64,
    name: &str,
    class: AssetClass,
    listed_enabled: Option<bool>,
    s: &ProtoOaSymbol,
    read_at: DateTime<Utc>,
) -> SymbolRow {
    let mut r = Map::new();
    let trading_mode = enum_name(s.trading_mode, |e: ProtoOaTradingMode| e.as_str_name());
    note_null(
        &mut r,
        "listed_enabled",
        "ProtoOALightSymbol.enabled",
        &listed_enabled,
    );
    note_null(
        &mut r,
        "trading_mode",
        "tradingMode (proto2 default ENABLED)",
        &trading_mode,
    );
    note_null(&mut r, "lot_size", "lotSize", &s.lot_size);
    note_null(&mut r, "min_volume", "minVolume", &s.min_volume);
    note_null(&mut r, "max_volume", "maxVolume", &s.max_volume);
    note_null(&mut r, "volume_step", "stepVolume", &s.step_volume);
    note_null(
        &mut r,
        "schedule_time_zone",
        "scheduleTimeZone",
        &s.schedule_time_zone,
    );
    note_null(
        &mut r,
        "measurement_units",
        "measurementUnits",
        &s.measurement_units,
    );
    r.insert(
        "value_per_point".into(),
        json!("not given by ProtoOASymbol; needs lot size, price and a conversion rate (phase 4)"),
    );
    SymbolRow {
        account_id,
        broker_symbol_id: s.symbol_id,
        name: name.to_string(),
        asset_class: class.as_str().to_string(),
        listed_enabled,
        trading_mode,
        digits: s.digits,
        pip_position: s.pip_position,
        lot_size: s.lot_size,
        min_volume: s.min_volume,
        max_volume: s.max_volume,
        volume_step: s.step_volume,
        schedule_time_zone: s.schedule_time_zone.clone(),
        measurement_units: s.measurement_units.clone(),
        null_reasons: Value::Object(r),
        raw: s.encode_to_vec(),
        read_at,
    }
}

pub fn cost_row(
    account_id: i64,
    symbol_id: i64,
    run_id: i64,
    measured_at: DateTime<Utc>,
    window: Duration,
    sample: Option<&SpreadSamples>,
    sym: Option<&ProtoOaSymbol>,
) -> CostRow {
    let mut r = Map::new();
    let spread_median = sample.and_then(|s| s.median());
    let spread_samples = sample.map(|s| s.spreads.len()).unwrap_or(0) as i32;
    if spread_median.is_none() {
        r.insert(
            "spread_median".into(),
            json!(format!(
                "no quote with both bid and ask in the {} s window (market closed or no quotes)",
                window.as_secs()
            )),
        );
    }
    r.insert(
        "commission_per_lot_side".into(),
        json!("not derived in phase 1: the broker gives preciseTradingCommissionRate in units of commissionType; money per lot needs prices and conversion rates"),
    );
    let market = match sym {
        Some(s) => market_state_of(s, measured_at),
        None => jiaoyi_core::schedule::MarketState::Unknown("symbol details not read".into()),
    };
    let mut row = CostRow {
        account_id,
        broker_symbol_id: symbol_id,
        load_run_id: Some(run_id),
        measured_at,
        market_open: market.is_open(),
        market_state_reason: market.reason().map(String::from),
        spread_median,
        spread_samples,
        spread_window_secs: window.as_secs() as i32,
        source: "ctrader ProtoOASymbol + ProtoOASpotEvent".into(),
        ..Default::default()
    };
    match sym {
        None => {
            for c in [
                "commission_type",
                "precise_trading_commission_rate",
                "commission_deprecated",
                "precise_min_commission",
                "min_commission_type",
                "min_commission_asset",
                "swap_long",
                "swap_short",
                "swap_calculation_type",
                "swap_rollover_3days",
                "swap_period_hours",
                "swap_time_minutes",
            ] {
                r.insert(c.into(), json!("symbol details not read"));
            }
        }
        Some(s) => {
            row.commission_type = enum_name(s.commission_type, |e: ProtoOaCommissionType| {
                e.as_str_name()
            });
            row.precise_trading_commission_rate = s.precise_trading_commission_rate;
            #[allow(deprecated)]
            {
                row.commission_deprecated = s.commission;
            }
            row.precise_min_commission = s.precise_min_commission;
            row.min_commission_type =
                enum_name(s.min_commission_type, |e: ProtoOaMinCommissionType| {
                    e.as_str_name()
                });
            row.min_commission_asset = s.min_commission_asset.clone();
            row.swap_long = s.swap_long;
            row.swap_short = s.swap_short;
            row.swap_calculation_type =
                enum_name(s.swap_calculation_type, |e: ProtoOaSwapCalculationType| {
                    e.as_str_name()
                });
            row.swap_rollover_3days =
                enum_name(s.swap_rollover3_days, |e: ProtoOaDayOfWeek| e.as_str_name());
            row.swap_period_hours = s.swap_period;
            row.swap_time_minutes = s.swap_time;
            note_null(
                &mut r,
                "commission_type",
                "commissionType (proto2 default USD_PER_MILLION_USD)",
                &row.commission_type,
            );
            note_null(
                &mut r,
                "precise_trading_commission_rate",
                "preciseTradingCommissionRate",
                &row.precise_trading_commission_rate,
            );
            note_null(
                &mut r,
                "commission_deprecated",
                "commission (deprecated)",
                &row.commission_deprecated,
            );
            note_null(
                &mut r,
                "precise_min_commission",
                "preciseMinCommission",
                &row.precise_min_commission,
            );
            note_null(
                &mut r,
                "min_commission_type",
                "minCommissionType (proto2 default CURRENCY)",
                &row.min_commission_type,
            );
            note_null(
                &mut r,
                "min_commission_asset",
                "minCommissionAsset (proto2 default USD)",
                &row.min_commission_asset,
            );
            note_null(&mut r, "swap_long", "swapLong", &row.swap_long);
            note_null(&mut r, "swap_short", "swapShort", &row.swap_short);
            note_null(
                &mut r,
                "swap_calculation_type",
                "swapCalculationType (proto2 default PIPS)",
                &row.swap_calculation_type,
            );
            note_null(
                &mut r,
                "swap_rollover_3days",
                "swapRollover3Days (proto2 default MONDAY)",
                &row.swap_rollover_3days,
            );
            note_null(
                &mut r,
                "swap_period_hours",
                "swapPeriod",
                &row.swap_period_hours,
            );
            note_null(
                &mut r,
                "swap_time_minutes",
                "swapTime",
                &row.swap_time_minutes,
            );
        }
    }
    row.null_reasons = Value::Object(r);
    row
}

/// Whether the market was open at `at`, from the symbol's own schedule.
pub fn market_state_of(s: &ProtoOaSymbol, at: DateTime<Utc>) -> jiaoyi_core::schedule::MarketState {
    use jiaoyi_core::schedule::{market_state, Holiday, Interval};
    let intervals: Vec<Interval> = s
        .schedule
        .iter()
        .map(|i| Interval {
            start_second: i.start_second,
            end_second: i.end_second,
        })
        .collect();
    let holidays: Vec<Holiday> = s
        .holiday
        .iter()
        .map(|h| Holiday {
            name: h.name.clone(),
            schedule_time_zone: h.schedule_time_zone.clone(),
            date_days: h.holiday_date,
            recurring: h.is_recurring,
            start_second: h.start_second,
            end_second: h.end_second,
        })
        .collect();
    market_state(&intervals, &holidays, s.schedule_time_zone.as_deref(), at)
}

/// The tradability of a stored symbol, for the verdict: listed as enabled and
/// in trading mode ENABLED (an absent tradingMode is the proto2 default ENABLED).
pub fn tradability(
    listed_enabled: Option<bool>,
    trading_mode: Option<&str>,
) -> (Option<bool>, Option<String>) {
    let mode = trading_mode.unwrap_or("ENABLED");
    match listed_enabled {
        Some(false) => (
            Some(false),
            Some("the symbol list marks it not enabled".into()),
        ),
        _ if mode != "ENABLED" => (Some(false), Some(format!("tradingMode {mode}"))),
        Some(true) => (Some(true), None),
        None => (None, None),
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Open a TLS connection for an environment.
pub async fn connect(
    broker: &BrokerConfig,
    env: jiaoyi_core::allowlist::Environment,
) -> Result<Connection, ClientError> {
    Connection::connect_tls(broker.host(env), broker.port).await
}
