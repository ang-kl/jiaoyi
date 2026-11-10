//! The edge test's pure computations (pre-registration of 24-10-2026,
//! docs/preregistration-2026-10-24.md). No I/O: the service feeds these the
//! stored bars and costs, and stores what they return.
//!
//! - [`rng`]: the seeded generator every random draw uses (seed logged).
//! - [`stats`]: mean, profit factor, moments and order statistics.
//! - [`normal`]: the normal CDF and its inverse.
//! - [`dsr`]: the deflated Sharpe ratio (Bailey & López de Prado).
//! - [`bootstrap`]: the week-block bootstrap of trade R.
//! - [`h1`]: the H1 trend rules on daily bars.
//! - [`control`]: the random-entry control.
//! - [`money`]: notional money beside R.
//! - [`verdict`]: the PASS rules, the development gate and the verdicts.

pub mod bootstrap;
pub mod control;
pub mod dsr;
pub mod exit_replay;
pub mod h1;
pub mod money;
pub mod normal;
pub mod rng;
pub mod stats;
pub mod verdict;

pub mod s3_signal;

pub mod s3_ticks;

pub mod class_prediction;
pub mod fx_diagnostic;
pub mod opening_range;

pub mod production_s3;

pub mod seasonal;

pub mod native_target_sizing;
pub mod offline_repaired;
pub mod option_ab;
pub mod peer_evaluation;
mod repaired_exit;
pub mod three_year;

pub mod manifest_pullback;

pub mod tiered_pullback;

pub mod rebuild;
pub mod rebuild_continuity;

pub mod profit_lock;
