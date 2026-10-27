//! Read-only session preparation: start T-20, deadline T-10. No trade actions.
//! Source: owner chat №17,616 §1, 28-10-2026 SGT. The actual opening price,
//! executable quote and current account risk remain entry-time observations.

use super::*;
use serde::{Deserialize, Serialize};

pub const START_MINUTES: i64 = 20;
pub const LEAD_MINUTES: i64 = 10;
const READ_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Clone)]
pub struct Spec {
    pub symbol: String,
    pub id: Option<i64>,
    pub rule: Option<SessionRule>,
    pub metadata_ready: bool,
}

pub async fn specs(
    pool: &PgPool,
    account: i64,
    syms: &HashMap<i64, Sym>,
    deposit: bool,
) -> anyhow::Result<Vec<Spec>> {
    let settings = jiaoyi_store::settings::snapshot(pool, Some(account)).await?;
    let universe = settings
        .universe("universe.symbols")
        .map_err(anyhow::Error::msg)?;
    Ok(universe
        .into_iter()
        .map(|(symbol, class)| {
            let sym = syms.values().find(|s| s.name == symbol);
            Spec {
                rule: sessions::rule_for(&symbol, class),
                id: sym.map(|s| s.id),
                metadata_ready: deposit
                    && sym.is_some_and(|s| {
                        s.quote.is_some() && s.limits.is_some_and(|l| l.min > 0 && l.step > 0)
                    }),
                symbol,
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prepared {
    pub symbol: String,
    pub symbol_id: Option<i64>,
    pub session: Session,
    pub prereg_hash: String,
    pub source_cutoff_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub table: Option<SigmaTable>,
    pub spread_ready: bool,
    pub commission_ready: bool,
    pub metadata_ready: bool,
    pub missing_marks: usize,
    pub source_error: bool,
}

impl Prepared {
    pub fn ready(&self) -> bool {
        self.table.is_some()
            && self.missing_marks == 0
            && self.spread_ready
            && self.commission_ready
            && self.metadata_ready
            && !self.source_error
    }
    pub fn on_time(&self) -> bool {
        self.ready()
            && self.completed_at <= self.session.open - chrono::Duration::minutes(LEAD_MINUTES)
    }
    pub fn matches(&self, symbol_id: i64, session: Session) -> bool {
        self.symbol_id == Some(symbol_id)
            && self.session == session
            && self.prereg_hash == prereg_hash()
    }
}

/// Entry marks only; lunch and the last 30 minutes remain excluded by the
/// registered rule. Missing data is retained, never replaced by a target.
fn missing_marks(table: &SigmaTable, session: Session) -> usize {
    (30..(session.close - session.open).num_minutes())
        .step_by(30)
        .filter(|m| {
            band::entry_window(&session, session.open + chrono::Duration::minutes(*m)).is_ok()
        })
        .filter(|m| {
            table
                .marks
                .get(m)
                .and_then(|v| v.as_ref().ok())
                .is_none_or(|x| !x.is_finite() || *x <= 0.0)
        })
        .count()
}

async fn stored(
    pool: &PgPool,
    account: i64,
    symbol: &str,
    session: Session,
) -> anyhow::Result<Option<Prepared>> {
    let value:Option<Value>=jiaoyi_store::sqlx::query_scalar(
        "SELECT detail FROM trading_decisions WHERE account_id=$1 AND symbol=$2 AND kind='s3_preopen'
         AND detail->>'prereg_hash'=$3 AND detail->'session'->>'open'=$4 ORDER BY id DESC LIMIT 1")
        .bind(account).bind(symbol).bind(prereg_hash()).bind(session.open.to_rfc3339_opts(chrono::SecondsFormat::Secs,true))
        .fetch_optional(pool).await?;
    Ok(value
        .and_then(|v| serde_json::from_value::<Prepared>(v).ok())
        .filter(|p| p.session == session && p.symbol == symbol && p.prereg_hash == prereg_hash()))
}

/// Hydrate already completed work before subscribing to entry quotes.
pub async fn restore(
    pool: &PgPool,
    account: i64,
    syms: &mut HashMap<i64, Sym>,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    for sym in syms.values_mut() {
        let Some(rule) = sym.rule else { continue };
        for session in sessions::session_at(&rule, now)
            .into_iter()
            .chain(sessions::next_session(&rule, now))
        {
            if let Some(saved) = stored(pool, account, &sym.name, session)
                .await?
                .filter(|p| {
                    p.symbol_id == Some(sym.id)
                        && p.completed_at <= now
                        && p.source_cutoff_at <= p.completed_at
                })
            {
                sym.preopen = Some(saved);
                break;
            }
        }
    }
    Ok(())
}

/// One worker per existing account connection. The actor polls this future
/// beside its quote/exit branches; there is no mutable trader in this task.
pub async fn run(
    pool: &PgPool,
    connection: &AccountSession<'_>,
    specs: Vec<Spec>,
    tx: tokio::sync::watch::Sender<HashMap<String, Prepared>>,
    mut stop: Shutdown,
    offset: chrono::Duration,
) -> anyhow::Result<()> {
    loop {
        if run_attempt(
            pool,
            connection,
            specs.clone(),
            tx.clone(),
            stop.clone(),
            offset,
        )
        .await
        .is_ok()
        {
            return Ok(());
        }
        // Optional preparation failure never takes the trading actor down.
        tracing::warn!(
            account = connection.account().id(),
            "pre-open source failure; preparation will retry"
        );
        tokio::select! {_=stop.wait()=>return Ok(()),_=tokio::time::sleep(Duration::from_secs(30))=>{}}
    }
}

async fn run_attempt(
    pool: &PgPool,
    connection: &AccountSession<'_>,
    specs: Vec<Spec>,
    tx: tokio::sync::watch::Sender<HashMap<String, Prepared>>,
    mut stop: Shutdown,
    offset: chrono::Duration,
) -> anyhow::Result<()> {
    let account = connection.account().id();
    let mut attempts: HashMap<String, Prepared> = HashMap::new();
    let mut tick = tokio::time::interval(Duration::from_secs(30));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! { _=stop.wait()=>return Ok(()), _=tick.tick()=>{} }
        let now = Utc::now() + offset;
        for spec in &specs {
            let Some(rule) = spec.rule else { continue };
            // Restore both the current and next session across reconnects.
            let current = sessions::session_at(&rule, now);
            let next = sessions::next_session(&rule, now);
            for session in current.into_iter().chain(next) {
                let key = format!("{}:{}", spec.symbol, session.open);
                if !attempts.contains_key(&key) {
                    if let Some(saved) =
                        stored(pool, account, &spec.symbol, session)
                            .await?
                            .filter(|p| {
                                p.symbol_id == spec.id
                                    && p.completed_at <= Utc::now() + offset
                                    && p.source_cutoff_at <= p.completed_at
                            })
                    {
                        attempts.insert(key.clone(), saved.clone());
                        tx.send_replace(attempts.clone());
                    }
                }
                let now = Utc::now() + offset;
                if session.open <= now
                    || now < session.open - chrono::Duration::minutes(START_MINUTES)
                {
                    continue;
                }
                if attempts.get(&key).is_some_and(|p| {
                    p.ready()
                        || now - p.completed_at < chrono::Duration::from_std(PREPARE_RETRY).unwrap()
                }) {
                    continue;
                }
                let commission =
                    match jiaoyi_store::research::commission_per_unit(pool, account).await {
                        Ok(samples) => costs::commission_book(&samples).is_ok(),
                        Err(_) => false,
                    };
                let cutoff = Utc::now() + offset;
                let read = async {
                    match spec.id {
                        Some(id) => {
                            load_day(pool, connection, account, id, rule, session, cutoff, None)
                                .await
                                .ok()
                        }
                        None => None,
                    }
                };
                let day = tokio::select! {
                    _=stop.wait()=>return Ok(()),
                    result=tokio::time::timeout(READ_TIMEOUT,read)=>result.ok().flatten(),
                };
                let p = Prepared {
                    symbol: spec.symbol.clone(),
                    symbol_id: spec.id,
                    session,
                    prereg_hash: prereg_hash(),
                    source_cutoff_at: cutoff,
                    completed_at: Utc::now() + offset,
                    missing_marks: day.as_ref().map_or(0, |d| missing_marks(&d.table, session)),
                    spread_ready: day.as_ref().is_some_and(|d| {
                        d.spread
                            .as_ref()
                            .is_ok_and(|(v, n)| v.is_finite() && *v >= 0.0 && *n > 0)
                    }),
                    commission_ready: commission,
                    metadata_ready: spec.metadata_ready,
                    source_error: day.is_none(),
                    table: day.map(|d| d.table),
                };
                let reason = if p.on_time() {
                    "pre-open inputs prepared by T-10"
                } else if p.ready() {
                    "pre-open inputs prepared late"
                } else {
                    "pre-open inputs incomplete"
                };
                // Only bounded coefficients/checks, no raw bars or account money.
                ts::record_decision(
                    pool,
                    account,
                    Some(&spec.symbol),
                    "s3_preopen",
                    reason,
                    &serde_json::to_value(&p)?,
                    None,
                )
                .await?;
                attempts.insert(key, p.clone());
                tx.send_replace(attempts.clone());
            }
        }
        attempts.retain(|_, p| p.session.close > Utc::now() + offset);
    }
}

/// Compact evidence for every configured symbol, including unresolved ones.
pub async fn health(pool: &PgPool, account: i64, now: DateTime<Utc>) -> anyhow::Result<Value> {
    let settings = jiaoyi_store::settings::snapshot(pool, Some(account)).await?;
    let universe = settings
        .universe("universe.symbols")
        .map_err(anyhow::Error::msg)?;
    let mut rows = Vec::new();
    for (symbol, class) in universe {
        let Some(rule) = sessions::rule_for(&symbol, class) else {
            rows.push(json!({"symbol":symbol,"state":"no_registered_session"}));
            continue;
        };
        for session in sessions::session_at(&rule, now)
            .into_iter()
            .chain(sessions::next_session(&rule, now))
        {
            let p = stored(pool, account, &symbol, session)
                .await?
                .filter(|p| p.completed_at <= now && p.source_cutoff_at <= p.completed_at);
            let deadline = session.open - chrono::Duration::minutes(LEAD_MINUTES);
            let state = match &p {
                Some(p) if p.on_time() => "prepared_on_time",
                Some(p) if p.ready() => "prepared_late",
                Some(_) => "incomplete",
                None if now > deadline => "deadline_missed",
                None if now >= session.open - chrono::Duration::minutes(START_MINUTES) => {
                    "awaiting_preparation"
                }
                None => "scheduled",
            };
            rows.push(json!({"symbol":symbol,"session_open":session.open,"session_close":session.close,
                "session_relation":if now>=session.open {"current"}else{"upcoming"},
                "preparation_starts_at":session.open-chrono::Duration::minutes(START_MINUTES),"deadline_at":deadline,
                "state":state,"completed_at":p.as_ref().map(|p|p.completed_at),
                "deadline_met":p.as_ref().map(Prepared::on_time),
                "history_ready":p.as_ref().map(|p|p.table.is_some() && p.missing_marks==0),
                "spread_ready":p.as_ref().map(|p|p.spread_ready),"commission_ready":p.as_ref().map(|p|p.commission_ready),
                "metadata_ready":p.as_ref().map(|p|p.metadata_ready),"missing_marks":p.as_ref().and_then(|p|p.table.as_ref().map(|_|p.missing_marks)),
                "source_cutoff_at":p.as_ref().map(|p|p.source_cutoff_at)}));
        }
    }
    Ok(
        json!({"start_minutes_before_open":START_MINUTES,"deadline_minutes_before_open":LEAD_MINUTES,
        "clock":"registered exchange timezone with DST; UTC session for crypto and FX/metals/energy",
        "scope":"pre-open inputs; actual opening price, cost/R, Kelly, fresh equity/margin and ownership remain entry-time checks",
        "symbols":rows}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::collections::BTreeMap;

    fn session() -> Session {
        sessions::session_on(
            &sessions::rule_for("BTCUSD", AssetClass::Crypto).unwrap(),
            NaiveDate::from_ymd_opt(2026, 10, 28).unwrap(),
        )
        .unwrap()
    }
    fn complete() -> Prepared {
        let session = session();
        Prepared {
            symbol: "BTCUSD".into(),
            symbol_id: Some(22),
            session,
            prereg_hash: prereg_hash(),
            source_cutoff_at: session.open - chrono::Duration::minutes(20),
            completed_at: session.open - chrono::Duration::minutes(10),
            table: Some(SigmaTable {
                marks: (30..=1440)
                    .step_by(30)
                    .map(|m| (m, Ok(0.01)))
                    .collect::<BTreeMap<_, _>>(),
                sessions_used: vec![],
            }),
            missing_marks: 0,
            spread_ready: true,
            commission_ready: true,
            metadata_ready: true,
            source_error: false,
        }
    }
    #[test]
    fn completion_after_deadline_is_late_even_when_the_read_started_early() {
        let mut p = complete();
        assert!(p.on_time());
        p.completed_at += chrono::Duration::milliseconds(1);
        assert!(p.ready());
        assert!(!p.on_time());
        p.metadata_ready = false;
        assert!(!p.ready());
    }
    #[test]
    fn absent_eligible_coefficients_are_not_certified_by_an_empty_table() {
        let mut p = complete();
        assert_eq!(missing_marks(p.table.as_ref().unwrap(), p.session), 0);
        p.table.as_mut().unwrap().marks.remove(&30);
        assert_eq!(missing_marks(p.table.as_ref().unwrap(), p.session), 1);
        p.table.as_mut().unwrap().marks.clear();
        assert_eq!(missing_marks(p.table.as_ref().unwrap(), p.session), 46);
    }
    #[test]
    fn deadlines_follow_registered_exchange_dst_and_crypto_utc_rollover() {
        for (symbol, class, date, open) in [
            (
                "US500",
                AssetClass::EquityIndex,
                (2026, 10, 28),
                "2026-10-28T13:30:00Z",
            ),
            (
                "US500",
                AssetClass::EquityIndex,
                (2026, 11, 3),
                "2026-11-03T14:30:00Z",
            ),
            (
                "GER40",
                AssetClass::EquityIndex,
                (2026, 10, 28),
                "2026-10-28T08:00:00Z",
            ),
            (
                "AUS200",
                AssetClass::EquityIndex,
                (2026, 10, 28),
                "2026-10-27T23:00:00Z",
            ),
            (
                "JPN225",
                AssetClass::EquityIndex,
                (2026, 10, 28),
                "2026-10-28T00:00:00Z",
            ),
            (
                "XAUUSD",
                AssetClass::Metal,
                (2026, 10, 28),
                "2026-10-28T07:00:00Z",
            ),
            (
                "SpotCrude",
                AssetClass::Energy,
                (2026, 10, 28),
                "2026-10-28T07:00:00Z",
            ),
            (
                "BTCUSD",
                AssetClass::Crypto,
                (2026, 10, 28),
                "2026-10-28T00:00:00Z",
            ),
        ] {
            let rule = sessions::rule_for(symbol, class).unwrap();
            let date = NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap();
            let s = sessions::session_on(&rule, date).unwrap();
            let expected: DateTime<Utc> = open.parse().unwrap();
            assert_eq!(s.open, expected);
            assert_eq!(
                s.open - chrono::Duration::minutes(LEAD_MINUTES),
                expected - chrono::Duration::seconds(600)
            );
            assert_eq!(
                sessions::next_session(&rule, s.open - chrono::Duration::minutes(START_MINUTES)),
                Some(s)
            );
        }
    }
}
