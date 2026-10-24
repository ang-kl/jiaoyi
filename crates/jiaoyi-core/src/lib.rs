//! Pure rules shared by every jiaoyi crate. No I/O lives here.
//!
//! - [`allowlist`]: the account allowlist (`JIAOYI_ALLOWED_ACCOUNTS`, rulebook §2).
//! - [`settings`]: setting definitions, bounds checks and per-account overrides.
//! - [`universe`]: exact-name resolution of the `universe.symbols` setting.
//! - [`indicators`]: ATR with Wilder smoothing (rulebook §5).
//! - [`verdict`]: the phase-1 keep/drop checks (rulebook §3).
//! - [`bars`]: tick bars and tick-imbalance bars from quote updates.
//! - [`schedule`]: whether a market was open, from the broker's schedule.
//! - [`secret`]: a wrapper that never prints its contents.

pub mod allowlist;
pub mod bars;
pub mod indicators;
pub mod schedule;
pub mod secret;
pub mod settings;
pub mod universe;
pub mod verdict;
