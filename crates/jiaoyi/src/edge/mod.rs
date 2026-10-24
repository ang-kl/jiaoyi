//! The research engine for the edge test (R1): the pre-registered H1 jobs.
//!
//! The rules are `docs/preregistration-2026-10-24.md`, embedded here and
//! hashed at start ([`prereg`]); every job and trial carries that hash.
//!
//! - [`data`]: the one data-access layer per job stage. A development job
//!   cannot read any bar or tick on or after the held-out seal (2025-10-24):
//!   it is refused, never filtered silently.
//! - [`costs`]: spread, bar quote basis, commission, swap and FX, each
//!   labelled measured or estimated.
//! - [`sampler`]: the spread samples (one June week a year per symbol),
//!   streamed from the broker's tick history and never stored raw.
//! - [`h1job`]: one H1 job: the simulation, the control, the bootstrap and
//!   the verdict, written as one trial.
//! - [`recheck`]: held-out trades re-checked tick by tick.
//! - [`runner`]: the background runner; the held-out gate.
//! - [`routes`]: `/research/jobs` and `/research/trials` (counts and R only).

/// The version of the engine whose results a trial counts, compiled in and
/// recorded with every attempt and trial. It goes up whenever a change
/// alters what a trial would count.
/// - 1: the engine merged in #9 (9207eac1); attempts before this field
///   record none.
/// - 2: R1b, the held-out tick re-check checks the exit-side quote standing
///   at the fill before any later update (Codex P2 on #9, round 4).
pub const ENGINE_VERSION: i32 = 2;

/// The held-out job runs only on this engine or a later one (owner,
/// "Merge now, fix in next PR": the held-out stage never runs on an engine
/// without the round-4 fix). An older engine leaves it pending, not
/// attempted.
pub const HELD_OUT_MIN_ENGINE: i32 = 2;

pub mod costs;
pub mod data;
pub mod h1job;
pub mod prereg;
pub mod recheck;
pub mod routes;
pub mod runner;
pub mod sampler;
