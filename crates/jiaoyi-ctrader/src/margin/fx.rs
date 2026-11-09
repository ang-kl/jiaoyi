//! Account-scoped conversion identities and executable bid/ask bounds.
//! Broker conversion chains supply identity; historical bars never stand in
//! for a fresh rate. The final writer revalues foreign stop exposure again.
use crate::{entry_guard::Quote, pb};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MAX_FX_AGE_MS: i64 = 5_000;

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
pub fn conversion_bounds<'a>(
    path: &[ConversionLeg],
    from: i64,
    to: i64,
    now: DateTime<Utc>,
    quote: impl Fn(i64) -> Option<&'a Quote>,
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
        let q = quote(leg.symbol_id).ok_or_else(|| {
            format!(
                "conversion quote missing for broker symbol {}",
                leg.symbol_id
            )
        })?;
        let valid = |side: Option<(f64, DateTime<Utc>)>| {
            side.filter(|(p, at)| {
                let age = now.signed_duration_since(*at).num_milliseconds();
                p.is_finite() && *p > 0.0 && (0..=MAX_FX_AGE_MS).contains(&age)
            })
            .map(|(p, _)| p)
        };
        let (Some(bid), Some(ask)) = (valid(q.bid), valid(q.ask)) else {
            return Err(format!(
                "conversion quote stale/incomplete for broker symbol {}",
                leg.symbol_id
            ));
        };
        if bid > ask {
            return Err("conversion quote crossed".into());
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

pub fn validate_cached_fx<'a>(
    proof: &CachedGuard,
    now: DateTime<Utc>,
    quote: impl Fn(i64) -> Option<&'a Quote>,
) -> Result<(f64, f64), String> {
    if proof.account_id <= 0 {
        return Err("foreign exposure account invalid".into());
    }
    if let Some(limit) = proof.candidate_max_risk {
        if !limit.is_finite() || limit <= 0.0 {
            return Err("prepared candidate cash-risk limit invalid".into());
        }
        let risk = foreign_reserve(std::slice::from_ref(&proof.candidate_exposure), now, &quote)?;
        if risk > limit * (1.0 + 1e-9) {
            return Err(format!(
                "current conversion makes candidate risk {risk:.6} exceed policy {limit:.6}"
            ));
        }
    }
    super::evaluate(
        proof.equity,
        proof.projected_margin,
        proof.base_loss_reserve + foreign_reserve(&proof.foreign_exposures, now, quote)?,
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
