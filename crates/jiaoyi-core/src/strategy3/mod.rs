//! Strategy 3, intraday momentum (owner-approved rules, 25-10-2026): the
//! pure parts.
//!
//! - [`sessions`]: each symbol's sessions in UTC, with daylight saving time
//!   from the IANA database;
//! - [`band`]: the noise band, open × (1 ± σ), at any instant;
//! - [`targets`]: the exits, bot-trade's `risk_coverage_partial_v1`.
//!
//! The cost reserve and the slippage it charges are here too.

pub mod band;
pub mod sessions;
pub mod targets;

use crate::universe::AssetClass;

/// Per-side slippage, in basis points of price, by class: bot-trade's
/// `agent/config/momentum-target-slippage.json` (owner-confirmed 02-10-2026,
/// reply to № 10,585·B·5). bot-trade's classes map as: equity_index →
/// `index_cfd`, fx → `fx`, metal and energy → `commodity`, crypto →
/// `crypto`. Every class there is 24 bps (three of them charged the dearest
/// measured class for too few rows).
pub fn slippage_bps_per_side(class: AssetClass) -> f64 {
    match class {
        AssetClass::EquityIndex => 24.0, // index_cfd
        AssetClass::Fx => 24.0,          // fx
        AssetClass::Metal => 24.0,       // commodity
        AssetClass::Energy => 24.0,      // commodity
        AssetClass::Crypto => 24.0,      // crypto
    }
}

/// The cost reserve C per unit, in price units: the median spread, the
/// round-trip commission per unit and the slippage of both sides at `price`.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct CostReserve {
    pub spread: f64,
    pub commission_round_trip: f64,
    pub slippage: f64,
    pub total: f64,
}

pub fn cost_reserve(
    median_spread: f64,
    commission_round_trip: f64,
    slippage_bps_per_side: f64,
    price: f64,
) -> Result<CostReserve, String> {
    for (name, v) in [
        ("median spread", median_spread),
        ("round-trip commission", commission_round_trip),
        ("slippage", slippage_bps_per_side),
        ("price", price),
    ] {
        if !(v.is_finite() && v >= 0.0) {
            return Err(format!("the {name} is not known ({v})"));
        }
    }
    let slippage = 2.0 * slippage_bps_per_side / 10_000.0 * price;
    Ok(CostReserve {
        spread: median_spread,
        commission_round_trip,
        slippage,
        total: median_spread + commission_round_trip + slippage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reserve_adds_spread_commission_and_both_sides_slippage() {
        // US500 at 6000: spread 0.5, commission 0 per unit, 24 bps a side
        // → 2 × 0.0024 × 6000 = 28.8.
        let c = cost_reserve(
            0.5,
            0.0,
            slippage_bps_per_side(AssetClass::EquityIndex),
            6000.0,
        )
        .unwrap();
        assert!((c.slippage - 28.8).abs() < 1e-9);
        assert!((c.total - 29.3).abs() < 1e-9);
        assert!(cost_reserve(f64::NAN, 0.0, 24.0, 1.0).is_err());
        assert!(cost_reserve(0.1, -0.1, 24.0, 1.0).is_err());
    }
}
