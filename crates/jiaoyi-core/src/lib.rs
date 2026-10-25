//! Pure rules shared by every jiaoyi crate. No I/O lives here.
//!
//! - [`allowlist`]: the account allowlist (`JIAOYI_ALLOWED_ACCOUNTS`, rulebook §2).
//! - [`settings`]: setting definitions, bounds checks and per-account overrides.
//! - [`universe`]: exact-name resolution of the `universe.symbols` setting.
//! - [`indicators`]: ATR with Wilder smoothing (rulebook §5).
//! - [`verdict`]: the phase-1 keep/drop checks (rulebook §3).
//! - [`bars`]: tick bars and tick-imbalance bars from quote updates.
//! - [`schedule`]: whether a market was open, from the broker's schedule.
//! - [`seal`]: the pre-registration's held-out seal (2025-10-24).
//! - [`research`]: the edge test's pure computations (H1 simulation, control,
//!   bootstrap, deflated Sharpe, verdicts).
//! - [`secret`]: a wrapper that never prints its contents.
//! - [`strategy3`]: Strategy 3 (intraday momentum): sessions, the noise band
//!   and the exit plan.

pub mod allowlist;
pub mod bars;
pub mod indicators;
pub mod research;
pub mod schedule;
pub mod seal;
pub mod secret;
pub mod settings;
pub mod strategy3;
pub mod universe;
pub mod verdict;
