//! Strategy 3's exits: bot-trade's `risk_coverage_partial_v1`, ported line
//! by line from `agent/services/momentum-target-policy.js`
//! (`planMomentumTargets`, owner-approved for Strategy 3, 25-10-2026).
//!
//! With R the risk (entry to stop), C the cost reserve per unit and Q the
//! required reward multiple (`REQUIRED_RR`, 1.38 from 27-10-2026):
//! - TP1 (the trigger) = entry ± (Q·R + C), rounded outward to a whole tick;
//! - the close fraction at TP1 is f = (R + C) / (TP1 distance + R), the
//!   volume rounded up to the step and at least the broker minimum;
//! - the runner's target TP2 = TP1 ± R, sent as the broker take-profit;
//! - a position that cannot be split (the rest would fall below the
//!   minimum) closes whole at TP1, and the broker take-profit is TP1;
//! - a TP1 distance not above C is refused (`target_price_invalid`).
//!
//! The coverage identity: the part closed at TP1 pays for the runner's loss
//! at the original stop, after the reserve: close × (TP1 distance − C) ≥
//! runner × (R + C). The modelled fills are at the planned prices; gaps and
//! slippage may exceed the reserve.

use serde::Serialize;

/// bot-trade's policy name.
pub const POLICY: &str = "risk_coverage_partial_v1";
/// `MOMENTUM_MAX_PRICE_DIGITS`: a relative bracket carries at most 5 decimals.
pub const MAX_PRICE_DIGITS: i32 = 5;
/// `MOMENTUM_MIN_REQUIRED_RR`: TP1 sits Q·R + C from the entry.
///
/// **Owner, 27-10-2026: target near 1.4R.** Q = 1.38, the goal's payoff
/// reference. The previous hard minimum was 3.0. The 27 Oct statements
/// showed 3R + C (median C ≈ 2.2R, barrier ≈ 5R) cannot produce 73% wins
/// on a no-drift process. A further change of Q is a further registration.
/// See `docs/preregistration-2026-10-27-strategy3-v3.md`.
pub const REQUIRED_RR: f64 = 1.38;
const SNAP_TOLERANCE_TICKS: f64 = 1e-6;

/// `side`: buy or sell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Dir {
    Buy,
    Sell,
}

impl Dir {
    fn sign(self) -> f64 {
        match self {
            Dir::Buy => 1.0,
            Dir::Sell => -1.0,
        }
    }
}

/// The plan's inputs. Volumes are broker cents of units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Input {
    pub side: Dir,
    pub entry: f64,
    pub original_stop: f64,
    pub required_rr: f64,
    /// C: the cost reserve per unit, in price units.
    pub cost_reserve: f64,
    pub digits: i32,
    pub volume: i64,
    pub min_volume: i64,
    pub step_volume: i64,
}

/// `mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    PartialRunner,
    WholePositionMinimum,
}

/// The plan.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Plan {
    pub mode: Mode,
    pub initial_risk: f64,
    /// TP1.
    pub trigger: f64,
    /// The broker's take-profit: TP2 for a partial plan, TP1 for a whole one.
    pub broker_target: f64,
    /// TP2 (the runner's target), whatever the mode.
    pub runner_target: f64,
    pub volume: i64,
    pub close_volume: i64,
    pub runner_volume: i64,
    pub close_fraction: f64,
    pub modeled_net_at_original_stop: f64,
}

fn finite(n: f64) -> bool {
    n.is_finite()
}

fn positive(n: f64) -> bool {
    n.is_finite() && n > 0.0
}

fn price_digits(d: i32) -> bool {
    (0..=MAX_PRICE_DIGITS).contains(&d)
}

/// JavaScript's `Math.round` (half up), for the non-negative values used here.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// `priceTicks`: a price in whole ticks, or `None`.
pub fn price_ticks(price: f64, digits: i32) -> Option<i64> {
    if !finite(price) || !price_digits(digits) {
        return None;
    }
    let t = js_round(price * 10f64.powi(digits));
    (t.abs() < 9_007_199_254_740_991.0).then_some(t as i64)
}

/// `gridTicks`: whole ticks of a price within float residue of the grid.
fn grid_ticks(price: f64, digits: i32) -> Option<f64> {
    if !finite(price) || !price_digits(digits) {
        return None;
    }
    let scaled = price * 10f64.powi(digits);
    let nearest = js_round(scaled);
    ((scaled - nearest).abs() < SNAP_TOLERANCE_TICKS).then_some(nearest)
}

/// `outward`.
fn outward(price: f64, digits: i32, direction: f64) -> f64 {
    let factor = 10f64.powi(digits);
    let scaled = price * factor;
    let nearest = js_round(scaled);
    let integer = if (scaled - nearest).abs() < 1e-8 {
        nearest
    } else if direction == 1.0 {
        scaled.ceil()
    } else {
        scaled.floor()
    };
    integer / factor
}

/// `outwardTicks`.
fn outward_ticks(scaled: f64) -> f64 {
    let nearest = js_round(scaled);
    if (scaled - nearest).abs() < 1e-8 {
        nearest
    } else {
        scaled.ceil()
    }
}

/// `targets`: (trigger, runner target).
fn targets(i: &Input, risk: f64, direction: f64) -> (f64, f64) {
    let Some(entry_ticks) = grid_ticks(i.entry, i.digits) else {
        let trigger = outward(
            i.entry + direction * (i.required_rr * risk + i.cost_reserve),
            i.digits,
            direction,
        );
        return (
            trigger,
            outward(trigger + direction * risk, i.digits, direction),
        );
    };
    let factor = 10f64.powi(i.digits);
    let risk_ticks = match grid_ticks(i.original_stop, i.digits) {
        None => risk * factor,
        Some(stop_ticks) => direction * (entry_ticks - stop_ticks),
    };
    let trigger_ticks = outward_ticks(i.required_rr * risk_ticks + i.cost_reserve * factor);
    (
        (entry_ticks + direction * trigger_ticks) / factor,
        (entry_ticks + direction * (trigger_ticks + outward_ticks(risk_ticks))) / factor,
    )
}

/// `planMomentumTargets`. `Err` is bot-trade's refusal reason.
pub fn plan(i: &Input) -> Result<Plan, &'static str> {
    if !positive(i.entry)
        || !positive(i.original_stop)
        || !positive(i.required_rr)
        || !finite(i.cost_reserve)
        || i.cost_reserve < 0.0
    {
        return Err("risk_and_cost_evidence_required");
    }
    if i.required_rr < REQUIRED_RR {
        return Err("required_rr_below_hard_minimum");
    }
    if !(0..=8).contains(&i.digits) {
        return Err("price_precision_required");
    }
    if !price_digits(i.digits) {
        return Err("relative_bracket_precision_unsupported");
    }
    if i.volume <= 0
        || i.min_volume <= 0
        || i.step_volume <= 0
        || i.volume < i.min_volume
        || i.volume % i.step_volume != 0
    {
        return Err("broker_volume_invalid");
    }
    let direction = i.side.sign();
    let risk = direction * (i.entry - i.original_stop);
    if risk.is_nan() || risk <= 0.0 {
        return Err("original_stop_on_wrong_side");
    }
    let (trigger, runner_target) = targets(i, risk, direction);
    let distance = direction * (trigger - i.entry);
    if !positive(trigger) || distance.is_nan() || distance <= i.cost_reserve {
        return Err("target_price_invalid");
    }
    let fraction = (risk + i.cost_reserve) / (distance + risk);
    let step = i.step_volume as f64;
    let desired = ((i.volume as f64 * fraction).max(i.min_volume as f64) / step).ceil() as i64
        * i.step_volume;
    let partial = desired < i.volume && i.volume - desired >= i.min_volume;
    if partial && (!positive(runner_target) || direction * (runner_target - trigger) <= 0.0) {
        return Err("target_price_invalid");
    }
    let close_volume = if partial { desired } else { i.volume };
    let runner_volume = i.volume - close_volume;
    let modeled = close_volume as f64 * (distance - i.cost_reserve)
        - runner_volume as f64 * (risk + i.cost_reserve);
    if !modeled.is_finite() || modeled < -1e-7 {
        return Err("coverage_not_representable");
    }
    Ok(Plan {
        mode: if partial {
            Mode::PartialRunner
        } else {
            Mode::WholePositionMinimum
        },
        initial_risk: risk,
        trigger,
        broker_target: if partial { runner_target } else { trigger },
        runner_target,
        volume: i.volume,
        close_volume,
        runner_volume,
        close_fraction: close_volume as f64 / i.volume as f64,
        modeled_net_at_original_stop: modeled,
    })
}

#[cfg(test)]
mod tests {
    //! bot-trade's own test numbers (`momentum-target-policy.test.js`).
    use super::*;

    fn input() -> Input {
        Input {
            side: Dir::Buy,
            entry: 100.0,
            original_stop: 90.0,
            required_rr: 3.0,
            cost_reserve: 0.4,
            digits: 2,
            volume: 10_000,
            min_volume: 100,
            step_volume: 100,
        }
    }

    #[test]
    fn partial_volume_covers_modeled_runner_loss_after_cost_and_broker_rounding() {
        let p = plan(&input()).unwrap();
        assert_eq!(p.mode, Mode::PartialRunner);
        assert_eq!(p.trigger, 130.4);
        assert_eq!(p.broker_target, 140.4);
        assert_eq!(p.close_volume, 2600);
        assert_eq!(p.runner_volume, 7400);
        assert!(p.modeled_net_at_original_stop >= 0.0);
        let independent = 2600.0 * (30.4 - 0.4) - 7400.0 * (10.0 + 0.4);
        assert!((independent - p.modeled_net_at_original_stop).abs() < 1e-7);
    }

    #[test]
    fn short_mirrors_long() {
        let p = plan(&Input {
            side: Dir::Sell,
            original_stop: 110.0,
            ..input()
        })
        .unwrap();
        assert_eq!(p.trigger, 69.6);
        assert_eq!(p.broker_target, 59.6);
        assert_eq!(p.close_volume, 2600);
    }

    #[test]
    fn minimum_volume_cannot_create_dust_or_silently_enlarge_the_position() {
        let p = plan(&Input {
            volume: 100,
            min_volume: 100,
            ..input()
        })
        .unwrap();
        assert_eq!(p.mode, Mode::WholePositionMinimum);
        assert_eq!((p.close_volume, p.runner_volume), (100, 0));
        assert_eq!(p.broker_target, p.trigger);
        let exact = plan(&Input {
            volume: 200,
            ..input()
        })
        .unwrap();
        assert_eq!((exact.close_volume, exact.runner_volume), (100, 100));
    }

    #[test]
    fn missing_costs_invalid_units_and_a_crossed_stop_refuse() {
        let i = input();
        for bad in [
            Input {
                cost_reserve: f64::NAN,
                ..i
            },
            Input {
                cost_reserve: -1.0,
                ..i
            },
            Input { volume: 101, ..i },
            Input { min_volume: 0, ..i },
            Input {
                step_volume: 0,
                ..i
            },
            Input {
                original_stop: 105.0,
                ..i
            },
            Input {
                entry: f64::INFINITY,
                ..i
            },
            Input {
                side: Dir::Sell,
                original_stop: 200.0,
                ..i
            },
        ] {
            assert!(plan(&bad).is_err(), "{bad:?}");
        }
        assert_eq!(
            plan(&Input {
                required_rr: 1.37,
                ..i
            }),
            Err("required_rr_below_hard_minimum")
        );
        // Owner 27-10-2026: near 1.4R is the minimum, and it plans.
        assert!(plan(&Input {
            required_rr: 1.38,
            ..i
        })
        .is_ok());
        assert_eq!(
            plan(&Input { digits: 6, ..i }),
            Err("relative_bracket_precision_unsupported")
        );
        assert_eq!(
            plan(&Input { digits: -1, ..i }),
            Err("price_precision_required")
        );
        for d in 0..=5 {
            assert!(plan(&Input { digits: d, ..i }).is_ok(), "{d}");
        }
    }

    #[test]
    fn outward_rounding_never_lowers_the_floor() {
        for side in [Dir::Buy, Dir::Sell] {
            let stop = if side == Dir::Buy { 99.777 } else { 100.223 };
            let p = plan(&Input {
                side,
                original_stop: stop,
                cost_reserve: 0.0013,
                ..input()
            })
            .unwrap();
            let d = side.sign();
            assert!((d * (p.trigger - 100.0) - 0.0013) / 0.223 >= 3.0 - 1e-10);
            assert!(p.modeled_net_at_original_stop >= -1e-7);
        }
    }

    #[test]
    fn an_unsplittable_short_needs_only_its_whole_position_target() {
        let p = plan(&Input {
            side: Dir::Sell,
            original_stop: 125.0,
            volume: 100,
            ..input()
        })
        .unwrap();
        assert_eq!(p.mode, Mode::WholePositionMinimum);
        assert_eq!(p.broker_target, 24.6);
    }

    #[test]
    fn targets_move_by_exactly_the_slip_in_ticks() {
        let named = Input {
            side: Dir::Buy,
            required_rr: 3.0,
            cost_reserve: 0.4900000001,
            digits: 2,
            volume: 10_000,
            min_volume: 100,
            step_volume: 100,
            entry: 2904.25,
            original_stop: 2830.44,
        };
        let planned = plan(&named).unwrap();
        let filled = plan(&Input {
            entry: 2904.28,
            original_stop: 2830.47,
            ..named
        })
        .unwrap();
        assert_eq!(
            price_ticks(filled.trigger, 2).unwrap() - price_ticks(planned.trigger, 2).unwrap(),
            3
        );
        assert_eq!(
            price_ticks(filled.broker_target, 2).unwrap()
                - price_ticks(planned.broker_target, 2).unwrap(),
            3
        );
    }

    #[test]
    fn tick_space_targets_equal_price_rounding_off_the_tolerance() {
        let off_stop = plan(&Input {
            original_stop: 99.777,
            cost_reserve: 0.0013,
            ..input()
        })
        .unwrap();
        assert_eq!((off_stop.trigger, off_stop.broker_target), (100.68, 100.91));
        let off_entry = plan(&Input {
            entry: 100.0033,
            original_stop: 90.0,
            ..input()
        })
        .unwrap();
        assert_eq!(
            (off_entry.trigger, off_entry.broker_target),
            (130.42, 140.43)
        );
        let sell = plan(&Input {
            side: Dir::Sell,
            entry: 100.0033,
            original_stop: 110.0,
            ..input()
        })
        .unwrap();
        assert_eq!((sell.trigger, sell.broker_target), (69.61, 59.61));
    }
}
