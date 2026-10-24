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

pub mod costs;
pub mod data;
pub mod h1job;
pub mod prereg;
pub mod recheck;
pub mod routes;
pub mod runner;
pub mod sampler;
