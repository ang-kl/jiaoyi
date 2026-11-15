//! Account-scoped conversion identities and executable bid/ask bounds.
//! Broker conversion chains supply identity; historical bars never stand in
//! for a fresh rate. The final writer revalues foreign stop exposure again.
use crate::{entry_guard::Quote, pb};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MAX_FX_AGE_MS: i64 = 5_000;
pub const CLOSED_BOOK_POLICY: &str = "closed_book_fx_gap_5pct_v18462";
pub const CLOSED_BOOK_GAP_FACTOR: f64 = 1.05;

/// Accounting evidence only. Never installed into executable quote caches.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClosedBookQuote {
    pub account_id: i64,
    pub calendar: crate::market_hours::ContractCalendar,
    pub bid: f64,
    pub ask: f64,
    pub bid_at: DateTime<Utc>,
    pub ask_at: DateTime<Utc>,
    pub verified_at: DateTime<Utc>,
    pub source: String,
}
impl ClosedBookQuote {
    pub fn validate(&self, account: i64, now: DateTime<Utc>) -> Result<(), String> {
        let fail = || {
            "closed-book FX evidence invalid, expired, or outside its verified closure".to_string()
        };
        if account <= 0
            || self.account_id != account
            || self.source.is_empty()
            || !(0..=300_000).contains(&(now - self.verified_at).num_milliseconds())
            || !self.bid.is_finite()
            || !self.ask.is_finite()
            || self.bid <= 0.0
            || self.ask < self.bid
        {
            return Err(fail());
        }
        let (close, _) = self.calendar.closed_window(now).ok_or_else(fail)?;
        for at in [self.bid_at, self.ask_at] {
            if at >= close
                || (close - at).num_milliseconds() > MAX_FX_AGE_MS
                || self
                    .calendar
                    .validate(at, crate::trading::Side::Buy)
                    .is_err()
            {
                return Err(fail());
            }
        }
        Ok(())
    }
}

// Connection + account + revision isolation. Historical evidence cannot leak
// across reconnects, symbol metadata invalidation, or a changed account book.
type ClosedQuotes = std::sync::Mutex<BTreeMap<(u64, i64, u64, i64), ClosedBookQuote>>;
static CLOSED_QUOTES: std::sync::OnceLock<ClosedQuotes> = std::sync::OnceLock::new();
pub fn install_closed_book_quote(
    session: &crate::AccountSession<'_>,
    value: ClosedBookQuote,
) -> Result<(), String> {
    let account = session.account().id();
    value.validate(account, Utc::now())?;
    let conn = session.connection();
    let generation = conn.entry_generation();
    let revision = conn.entry_revision(account);
    let mut cache = CLOSED_QUOTES.get_or_init(Default::default).lock().unwrap();
    cache.retain(|(g, a, r, _), v| {
        (*a != account || (*g == generation && *r == revision))
            && (Utc::now() - v.verified_at).num_seconds() <= 300
    });
    cache.insert(
        (generation, account, revision, value.calendar.symbol_id),
        value,
    );
    Ok(())
}
pub fn closed_book_quotes(session: &crate::AccountSession<'_>) -> Vec<ClosedBookQuote> {
    let conn = session.connection();
    let key = (
        conn.entry_generation(),
        session.account().id(),
        conn.entry_revision(session.account().id()),
    );
    CLOSED_QUOTES
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .iter()
        .filter(|((g, a, r, _), v)| (*g, *a, *r) == key && v.validate(key.1, Utc::now()).is_ok())
        .map(|(_, v)| v.clone())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConversionLeg {
    pub symbol_id: i64,
    pub from_asset: i64,
    pub to_asset: i64,
    pub inverse: bool,
}

/// Validate a broker-returned conversion graph and choose its shortest chain.
/// Identity is based on asset IDs, never a symbol-name guess.
pub fn conversion_path(
    symbols: &[pb::ProtoOaLightSymbol],
    from: i64,
    to: i64,
) -> Result<Vec<ConversionLeg>, String> {
    if from <= 0 || to <= 0 {
        return Err("conversion asset identity invalid".into());
    }
    if from == to {
        return Ok(vec![]);
    }
    if symbols.is_empty() || symbols.len() > 32 {
        return Err("broker conversion chain missing or exceeds bounded graph".into());
    }
    let mut identities = BTreeMap::new();
    for s in symbols {
        let (Some(base), Some(quote)) = (s.base_asset_id, s.quote_asset_id) else {
            return Err("broker conversion symbol has incomplete asset identity".into());
        };
        if s.symbol_id <= 0
            || base <= 0
            || quote <= 0
            || base == quote
            || identities.insert(s.symbol_id, (base, quote)).is_some()
        {
            return Err("broker conversion identity invalid or duplicated".into());
        }
    }
    let mut queue = VecDeque::from([(from, Vec::new())]);
    let mut visited = BTreeSet::from([from]);
    while let Some((asset, path)) = queue.pop_front() {
        if path.len() >= 4 {
            continue;
        }
        for (&symbol_id, &(base, quote)) in &identities {
            let (next, inverse) = if asset == base {
                (quote, false)
            } else if asset == quote {
                (base, true)
            } else {
                continue;
            };
            if !visited.insert(next) {
                continue;
            }
            let mut path = path.clone();
            path.push(ConversionLeg {
                symbol_id,
                from_asset: asset,
                to_asset: next,
                inverse,
            });
            if next == to {
                return Ok(path);
            }
            queue.push_back((next, path));
        }
    }
    Err("broker conversion chain does not connect requested assets".into())
}

/// Conservative conversion interval: liability uses upper, profit uses lower.
fn live_prices(q: Option<&Quote>, now: DateTime<Utc>, id: i64) -> Result<(f64, f64), String> {
    let q = q.ok_or_else(|| format!("conversion quote missing for broker symbol {id}"))?;
    let valid = |side: Option<(f64, DateTime<Utc>)>| {
        side.filter(|(p, at)| {
            p.is_finite()
                && *p > 0.0
                && (0..=MAX_FX_AGE_MS).contains(&(now - *at).num_milliseconds())
        })
        .map(|(p, _)| p)
    };
    match (valid(q.bid), valid(q.ask)) {
        (Some(b), Some(a)) if b <= a => Ok((b, a)),
        _ => Err(format!(
            "conversion quote stale/incomplete/crossed for broker symbol {id}"
        )),
    }
}
fn chain_bounds(
    path: &[ConversionLeg],
    from: i64,
    to: i64,
    mut prices: impl FnMut(i64) -> Result<(f64, f64), String>,
) -> Result<(f64, f64), String> {
    if from <= 0 || to <= 0 || path.len() > 4 {
        return Err("conversion identity or path length invalid".into());
    }
    if from == to {
        return if path.is_empty() {
            Ok((1.0, 1.0))
        } else {
            Err("identity conversion has unexpected legs".into())
        };
    }
    let mut asset = from;
    let mut visited = BTreeSet::from([from]);
    let (mut low, mut high) = (1.0, 1.0);
    for leg in path {
        if leg.from_asset != asset || leg.symbol_id <= 0 || !visited.insert(leg.to_asset) {
            return Err("conversion chain identity is broken or cyclic".into());
        }
        let (bid, ask) = prices(leg.symbol_id)?;
        if !bid.is_finite() || !ask.is_finite() || bid <= 0.0 || ask < bid {
            return Err("conversion prices invalid".into());
        }
        if leg.inverse {
            low /= ask;
            high /= bid;
        } else {
            low *= bid;
            high *= ask;
        }
        asset = leg.to_asset;
    }
    if asset != to || !low.is_finite() || !high.is_finite() || low <= 0.0 || high < low {
        return Err("conversion chain result invalid or does not reach deposit asset".into());
    }
    Ok((low, high))
}
pub fn conversion_bounds<'a>(
    path: &[ConversionLeg],
    from: i64,
    to: i64,
    now: DateTime<Utc>,
    quote: impl Fn(i64) -> Option<&'a Quote>,
) -> Result<(f64, f64), String> {
    chain_bounds(path, from, to, |id| live_prices(quote(id), now, id))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FxExposure {
    pub quote_asset: i64,
    pub deposit_asset: i64,
    pub chain: Vec<ConversionLeg>,
    /// Already broker-reported gross P&L in deposit currency (zero for pending/new).
    pub gross_deposit: f64,
    /// Signed final cash P&L at the stop in quote currency; negative is a loss.
    pub stop_cash_quote: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CachedGuard {
    pub account_id: i64,
    pub equity: f64,
    pub projected_margin: f64,
    pub base_loss_reserve: f64,
    pub floor_pct: f64,
    pub warning_pct: f64,
    pub foreign_exposures: Vec<FxExposure>,
    /// Only existing open positions may use this reserve. Pending/new exposure stays live.
    pub existing_book_exposures: Vec<FxExposure>,
    pub closed_book_quotes: Vec<ClosedBookQuote>,
    pub conversion_calendars: Vec<crate::market_hours::ContractCalendar>,
    pub candidate_exposure: FxExposure,
    /// Bound by prepared admission to its exact cash-risk policy.
    pub candidate_max_risk: Option<f64>,
}

pub fn foreign_reserve<'a>(
    exposures: &[FxExposure],
    now: DateTime<Utc>,
    quote: impl Fn(i64) -> Option<&'a Quote>,
) -> Result<f64, String> {
    let mut reserve = 0.0;
    for x in exposures {
        if !x.gross_deposit.is_finite() || !x.stop_cash_quote.is_finite() {
            return Err("foreign exposure arithmetic invalid".into());
        }
        let (lower, upper) =
            conversion_bounds(&x.chain, x.quote_asset, x.deposit_asset, now, &quote)?;
        let rate = if x.stop_cash_quote >= 0.0 {
            lower
        } else {
            upper
        };
        reserve += (x.gross_deposit - x.stop_cash_quote * rate).max(0.0);
    }
    if reserve.is_finite() {
        Ok(reserve)
    } else {
        Err("foreign exposure reserve overflow".into())
    }
}

pub fn existing_book_reserve<'a>(
    account: i64,
    exposures: &[FxExposure],
    evidence: &[ClosedBookQuote],
    calendars: &[crate::market_hours::ContractCalendar],
    now: DateTime<Utc>,
    quote: impl Fn(i64) -> Option<&'a Quote>,
) -> Result<f64, String> {
    let mut total = 0.0;
    for x in exposures {
        if !x.gross_deposit.is_finite() || !x.stop_cash_quote.is_finite() {
            return Err("foreign exposure arithmetic invalid".into());
        }
        let mut closed = false;
        let (mut low, mut high) = chain_bounds(&x.chain, x.quote_asset, x.deposit_asset, |id| {
            let calendar = calendars
                .iter()
                .find(|c| c.symbol_id == id)
                .ok_or("existing-book FX calendar missing")?;
            if calendar.closed_window(now).is_some() {
                let e = evidence
                    .iter()
                    .find(|e| e.calendar.symbol_id == id)
                    .ok_or("closed-book pre-close FX quote missing")?;
                if e.calendar != *calendar {
                    return Err("closed-book FX calendar changed".into());
                }
                e.validate(account, now)?;
                closed = true;
                Ok((e.bid, e.ask))
            } else {
                calendar
                    .validate(now, crate::trading::Side::Buy)
                    .map_err(|e| e.to_string())?;
                live_prices(quote(id), now, id)
            }
        })?;
        if closed {
            low /= CLOSED_BOOK_GAP_FACTOR;
            high *= CLOSED_BOOK_GAP_FACTOR;
        }
        total += (x.gross_deposit
            - x.stop_cash_quote * if x.stop_cash_quote >= 0.0 { low } else { high })
        .max(0.0);
    }
    if total.is_finite() {
        Ok(total)
    } else {
        Err("closed book reserve overflow".into())
    }
}

pub fn validate_cached_fx<'a>(
    proof: &CachedGuard,
    now: DateTime<Utc>,
    quote: impl Fn(i64) -> Option<&'a Quote>,
) -> Result<(f64, f64), String> {
    if proof.account_id <= 0 {
        return Err("foreign exposure account invalid".into());
    }
    for leg in &proof.candidate_exposure.chain {
        proof
            .conversion_calendars
            .iter()
            .find(|c| c.symbol_id == leg.symbol_id)
            .ok_or("candidate FX calendar missing")?
            .validate(now, crate::trading::Side::Buy)
            .map_err(|e| format!("candidate FX market unavailable: {e}"))?;
    }
    let candidate_risk =
        foreign_reserve(std::slice::from_ref(&proof.candidate_exposure), now, &quote)?;
    if let Some(limit) = proof.candidate_max_risk {
        if !limit.is_finite() || limit <= 0.0 {
            return Err("prepared candidate cash-risk limit invalid".into());
        }
        let risk = candidate_risk;
        if risk > limit * (1.0 + 1e-9) {
            return Err(format!(
                "current conversion makes candidate risk {risk:.6} exceed policy {limit:.6}"
            ));
        }
    }
    super::evaluate(
        proof.equity,
        proof.projected_margin,
        proof.base_loss_reserve
            + foreign_reserve(&proof.foreign_exposures, now, &quote)?
            + existing_book_reserve(
                proof.account_id,
                &proof.existing_book_exposures,
                &proof.closed_book_quotes,
                &proof.conversion_calendars,
                now,
                &quote,
            )?,
        proof.floor_pct,
        proof.warning_pct,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn symbol(id: i64, base: i64, quote: i64) -> pb::ProtoOaLightSymbol {
        pb::ProtoOaLightSymbol {
            symbol_id: id,
            base_asset_id: Some(base),
            quote_asset_id: Some(quote),
            ..Default::default()
        }
    }
    fn quote(bid: f64, ask: f64, at: DateTime<Utc>) -> Quote {
        Quote {
            bid: Some((bid, at)),
            ask: Some((ask, at)),
        }
    }
    #[test]
    fn broker_identity_direct_inverse_and_cross_have_conservative_bounds() {
        let now = Utc::now();
        let usd_jpy = quote(150.0, 151.0, now);
        let eur_usd = quote(1.1, 1.2, now);
        let syms = vec![symbol(2, 1, 2), symbol(3, 3, 1)];
        let path = conversion_path(&syms, 2, 3).unwrap();
        let (low, high) = conversion_bounds(&path, 2, 3, now, |id| match id {
            2 => Some(&usd_jpy),
            3 => Some(&eur_usd),
            _ => None,
        })
        .unwrap();
        assert!((low - 1.0 / 151.0 / 1.2).abs() < 1e-12);
        assert!((high - 1.0 / 150.0 / 1.1).abs() < 1e-12);
        assert_eq!(
            conversion_bounds(&[], 1, 1, now, |_| None).unwrap(),
            (1.0, 1.0)
        );
        assert!(conversion_bounds(&[], 2, 1, now, |_| None).is_err());
        assert!(conversion_path(&[symbol(2, 1, 2), symbol(2, 1, 3)], 2, 1).is_err());
    }
    #[test]
    fn every_conversion_leg_requires_fresh_nonfuture_uncrossed_quotes() {
        let now = Utc::now();
        let path = conversion_path(&[symbol(2, 1, 2)], 2, 1).unwrap();
        for q in [
            quote(150.0, 151.0, now - chrono::Duration::milliseconds(5001)),
            quote(150.0, 151.0, now + chrono::Duration::milliseconds(1)),
            quote(152.0, 151.0, now),
        ] {
            assert!(conversion_bounds(&path, 2, 1, now, |_| Some(&q)).is_err());
        }
    }
    #[test]
    fn live_jpy_exposure_is_valued_with_usd_candidate_and_rechecked_at_writer() {
        let now = Utc::now();
        let chain = conversion_path(&[symbol(2, 1, 2)], 2, 1).unwrap();
        let mut proof = CachedGuard {
            account_id: 43002148,
            equity: 1000.0,
            projected_margin: 200.0,
            base_loss_reserve: 10.0,
            floor_pct: 250.0,
            warning_pct: 150.0,
            candidate_exposure: FxExposure {
                quote_asset: 1,
                deposit_asset: 1,
                chain: vec![],
                gross_deposit: 0.0,
                stop_cash_quote: -10.0,
            },
            candidate_max_risk: Some(10.0),
            existing_book_exposures: vec![],
            closed_book_quotes: vec![],
            conversion_calendars: vec![crate::market_hours::ContractCalendar::from_symbol(
                &pb::ProtoOaSymbol {
                    symbol_id: 2,
                    schedule_time_zone: Some("UTC".into()),
                    schedule: vec![pb::ProtoOaInterval {
                        start_second: 0,
                        end_second: 604800,
                    }],
                    ..Default::default()
                },
            )
            .unwrap()],
            foreign_exposures: vec![FxExposure {
                quote_asset: 2,
                deposit_asset: 1,
                chain,
                gross_deposit: 0.0,
                stop_cash_quote: -15_000.0,
            }],
        };
        let q = quote(150.0, 151.0, now);
        assert_eq!(
            foreign_reserve(&proof.foreign_exposures, now, |_| Some(&q)).unwrap(),
            100.0
        );
        assert_eq!(
            validate_cached_fx(&proof, now, |_| Some(&q)).unwrap(),
            (500.0, 445.0)
        );
        let adverse = quote(20.0, 21.0, now);
        assert!(validate_cached_fx(&proof, now, |_| Some(&adverse)).is_err());
        proof.foreign_exposures[0].gross_deposit = 200.0;
        proof.foreign_exposures[0].stop_cash_quote = 15_000.0;
        let reserve = foreign_reserve(&proof.foreign_exposures, now, |_| Some(&q)).unwrap();
        assert!((reserve - (200.0 - 15_000.0 / 151.0)).abs() < 1e-10);
        // Capacity can still pass while a newly adverse FX rate violates the
        // smaller individual-entry risk cap. Recheck both, not margin alone.
        proof.candidate_exposure = FxExposure {
            quote_asset: 2,
            deposit_asset: 1,
            chain: proof.foreign_exposures[0].chain.clone(),
            gross_deposit: 0.0,
            stop_cash_quote: -1500.0,
        };
        assert!(validate_cached_fx(&proof, now, |_| Some(&q)).is_ok());
        let changed = quote(140.0, 141.0, now);
        assert!(validate_cached_fx(&proof, now, |_| Some(&changed))
            .unwrap_err()
            .contains("candidate risk"));
    }
}

#[cfg(test)]
mod closed_book_tests {
    use super::*;
    use crate::market_hours::ContractCalendar;
    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }
    fn calendar(id: i64) -> ContractCalendar {
        ContractCalendar::from_symbol(&pb::ProtoOaSymbol {
            symbol_id: id,
            schedule_time_zone: Some("America/New_York".into()),
            schedule: vec![pb::ProtoOaInterval {
                start_second: 61260,
                end_second: 492900,
            }],
            ..Default::default()
        })
        .unwrap()
    }
    #[test]
    fn existing_closed_book_is_adversely_reserved_but_candidate_and_reopen_stay_live() {
        let now = at("2026-11-15T00:00:00Z");
        let close = at("2026-11-13T21:55:00Z");
        let c = calendar(5);
        let evidence = ClosedBookQuote {
            account_id: 7,
            calendar: c.clone(),
            bid: 0.65,
            ask: 0.66,
            bid_at: close - chrono::Duration::seconds(1),
            ask_at: close - chrono::Duration::seconds(1),
            verified_at: now,
            source: "test DB receipt".into(),
        };
        let exposure = FxExposure {
            quote_asset: 2,
            deposit_asset: 1,
            chain: vec![ConversionLeg {
                symbol_id: 5,
                from_asset: 2,
                to_asset: 1,
                inverse: false,
            }],
            gross_deposit: 0.0,
            stop_cash_quote: -100.0,
        };
        let mut proof = CachedGuard {
            account_id: 7,
            equity: 1000.0,
            projected_margin: 200.0,
            base_loss_reserve: 155.0 + 6.0,
            floor_pct: 250.0,
            warning_pct: 250.0,
            foreign_exposures: vec![],
            existing_book_exposures: vec![exposure.clone()],
            closed_book_quotes: vec![evidence.clone()],
            conversion_calendars: vec![c],
            candidate_exposure: FxExposure {
                quote_asset: 1,
                deposit_asset: 1,
                chain: vec![],
                gross_deposit: 0.0,
                stop_cash_quote: -6.0,
            },
            candidate_max_risk: Some(6.0),
        };
        let (_, stress) = validate_cached_fx(&proof, now, |_| None).unwrap();
        assert!((stress - (1000.0 - 161.0 - 69.3) / 2.0).abs() < 1e-9);
        let mut wrong = proof.clone();
        wrong.closed_book_quotes[0].account_id = 8;
        assert!(validate_cached_fx(&wrong, now, |_| None).is_err());
        let mut missing = proof.clone();
        missing.closed_book_quotes.clear();
        assert!(validate_cached_fx(&missing, now, |_| None).is_err());
        let mut old = proof.clone();
        old.closed_book_quotes[0].bid_at = close - chrono::Duration::seconds(6);
        assert!(validate_cached_fx(&old, now, |_| None).is_err());
        let mut unsafe_margin = proof.clone();
        unsafe_margin.equity = 700.0;
        assert!(validate_cached_fx(&unsafe_margin, now, |_| None).is_err());
        // Even a post-close subscription quote must never enable a foreign candidate.
        proof.candidate_exposure = exposure;
        let fresh = Quote {
            bid: Some((0.65, now)),
            ask: Some((0.66, now)),
        };
        assert!(validate_cached_fx(&proof, now, |_| Some(&fresh))
            .unwrap_err()
            .contains("candidate FX market"));
        proof.candidate_exposure = missing.candidate_exposure;
        proof.candidate_max_risk = None;
        let reopen = at("2026-11-15T22:01:00Z");
        assert!(validate_cached_fx(&proof, reopen, |_| Some(&fresh)).is_err());
        let current = Quote {
            bid: Some((0.65, reopen)),
            ask: Some((0.66, reopen)),
        };
        assert!(validate_cached_fx(&proof, reopen, |_| Some(&current)).is_ok());
        let weekday = at("2026-11-16T12:00:00Z");
        let stale = Quote {
            bid: Some((0.65, weekday - chrono::Duration::milliseconds(5001))),
            ask: Some((0.66, weekday)),
        };
        assert!(validate_cached_fx(&proof, weekday, |_| Some(&stale)).is_err());
    }
    #[test]
    fn inverse_profit_reserve_and_closure_boundaries_are_conservative() {
        let now = at("2026-11-15T00:00:00Z");
        let c = calendar(4);
        let (close, reopen) = c.closed_window(now).unwrap();
        assert_eq!(close, at("2026-11-13T21:55:00Z"));
        assert_eq!(reopen, at("2026-11-15T22:01:00Z"));
        assert!(c.closed_window(reopen).is_none());
        let summer = calendar(4)
            .closed_window(at("2026-07-11T12:00:00Z"))
            .unwrap();
        assert_eq!(summer.0, at("2026-07-10T20:55:00Z"));
        let e = ClosedBookQuote {
            account_id: 7,
            calendar: c.clone(),
            bid: 150.0,
            ask: 151.0,
            bid_at: close - chrono::Duration::seconds(1),
            ask_at: close - chrono::Duration::seconds(1),
            verified_at: now,
            source: "stored".into(),
        };
        let mut x = FxExposure {
            quote_asset: 2,
            deposit_asset: 1,
            chain: vec![ConversionLeg {
                symbol_id: 4,
                from_asset: 2,
                to_asset: 1,
                inverse: true,
            }],
            gross_deposit: 0.0,
            stop_cash_quote: -15000.0,
        };
        assert!(
            (existing_book_reserve(
                7,
                std::slice::from_ref(&x),
                std::slice::from_ref(&e),
                std::slice::from_ref(&c),
                now,
                |_| None
            )
            .unwrap()
                - 105.0)
                .abs()
                < 1e-9
        );
        x.gross_deposit = 200.0;
        x.stop_cash_quote = 15100.0;
        assert!(
            (existing_book_reserve(7, &[x], &[e], &[c], now, |_| None).unwrap()
                - (200.0 - 100.0 / 1.05))
                .abs()
                < 1e-9
        );
    }
}
