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
