//! Restart/partition adapter around the unchanged two-stage kernel.
//! Checkpoints contain identities and event offsets, never invented fills. A
//! restart reconstructs state from the verified journal prefix. File boundaries
//! do not call finish, amend a stop, or recalculate original risk.
use super::exit_replay::{Comparison, Costs, Coverage, Input, Quote};
use super::repaired_exit::TwoStageReplay;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub schema: String,
    pub engine_version: String,
    pub input_sha256: String,
    pub events: usize,
    pub event_chain_sha256: String,
}

pub struct ContinuousReplay {
    kernel: TwoStageReplay,
    checkpoint: Checkpoint,
}

fn digest<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

impl ContinuousReplay {
    pub fn new(input: Input, engine_version: &str) -> Result<Self, String> {
        if engine_version.is_empty() {
            return Err("engine identity required".into());
        }
        let checkpoint = Checkpoint {
            schema: "jiaoyi.continuous-replay.v1".into(),
            engine_version: engine_version.into(),
            input_sha256: digest(&input)?,
            events: 0,
            event_chain_sha256: digest(&"empty")?,
        };
        Ok(Self {
            kernel: TwoStageReplay::new(input).map_err(|e| format!("{e:?}"))?,
            checkpoint,
        })
    }

    pub fn push_partition(&mut self, quotes: &[Quote]) -> Result<(), String> {
        if self.checkpoint.events + quotes.len() > 2_000_000 {
            return Err("bounded replay event budget".into());
        }
        for quote in quotes {
            self.kernel.push(*quote).map_err(|e| format!("{e:?}"))?;
            self.checkpoint.event_chain_sha256 =
                digest(&(&self.checkpoint.event_chain_sha256, quote))?;
            self.checkpoint.events += 1;
        }
        Ok(())
    }

    pub fn checkpoint(&self) -> Checkpoint {
        self.checkpoint.clone()
    }

    pub fn restore(
        input: Input,
        engine_version: &str,
        checkpoint: &Checkpoint,
        immutable_prefix: &[Quote],
    ) -> Result<Self, String> {
        if checkpoint.events != immutable_prefix.len() {
            return Err("restart journal length mismatch".into());
        }
        let mut rebuilt = Self::new(input, engine_version)?;
        rebuilt.push_partition(immutable_prefix)?;
        if &rebuilt.checkpoint != checkpoint {
            return Err("restart code, original plan or journal mismatch".into());
        }
        Ok(rebuilt)
    }

    pub fn finish(self, coverage: Coverage, costs: Costs) -> Comparison {
        self.kernel.finish(coverage, costs)
    }
}

/// Canonical offline runner entry point: exercise a real retained quote path
/// across a partition and restart, against its original native outcome.
pub fn verify_recorded_path(
    source: &super::manifest_pullback::Input,
    input: Input,
    engine_version: &str,
) -> Result<serde_json::Value, String> {
    let (snapshots, _) = super::s3_ticks::snapshots(&source.request, &source.source)?;
    let end = source.request.to;
    let through = end + chrono::Duration::seconds(5);
    let quotes: Vec<_> = snapshots
        .iter()
        .filter(|q| {
            q.quote.at >= input.entered_at && q.quote.at <= through && (q.quote.at < end || q.fresh)
        })
        .map(|q| q.quote)
        .collect();
    if quotes.len() < 2 || input.session_end != end {
        return Err("recorded forced path or quote coverage missing".into());
    }
    // Compare generated Values inside Rust, before a JSON float parse can
    // introduce a one-ULP conversion difference. The caller also compares the
    // emitted outcome to its immutable historical receipt.
    let uninterrupted = super::option_ab::tick_run(&input, &snapshots, end, true)?;
    let split = quotes.len() / 2;
    let mut before = ContinuousReplay::new(input.clone(), engine_version)?;
    before.push_partition(&quotes[..split])?;
    let checkpoint = before.checkpoint();
    let mut after =
        ContinuousReplay::restore(input, engine_version, &checkpoint, &quotes[..split])?;
    after.push_partition(&quotes[split..])?;
    let final_checkpoint = after.checkpoint();
    let outcome = after
        .finish(
            Coverage {
                censored: true,
                ..Coverage::recorded_path(through)
            },
            Costs::default(),
        )
        .candidate;
    let value = serde_json::to_value(outcome).map_err(|e| e.to_string())?;
    if value != uninterrupted["outcome"] {
        return Err("partition/restart differs from uninterrupted native outcome".into());
    }
    Ok(
        serde_json::json!({"quotes":quotes.len(),"split_at":split,"checkpoint":final_checkpoint,
        "original_outcome_parity":true,"outcome":value,"financial_pass":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research::exit_replay::{Side, TrailCadence};
    use chrono::{Duration, TimeZone, Utc};

    fn input() -> Input {
        let at = Utc.with_ymd_and_hms(2024, 3, 8, 23, 59, 0).unwrap();
        Input {
            entered_at: at,
            fill: 100.,
            side: Side::Long,
            initial_r: 10.,
            volume: 10,
            tp1_close_volume: 10,
            tp1_distance: 13.8,
            broker_tp_distance: 13.8,
            session_end: at + Duration::hours(4),
            digits: 1,
            cadence: TrailCadence::S3ThirtySeconds,
        }
    }

    #[test]
    fn restart_across_midnight_preserves_pending_amend_and_bracket_priority() {
        let i = input();
        // +1R proposal in first partition, old initial stop wins after restart.
        let quotes: Vec<_> = [(30, 110.), (90, 89.)]
            .into_iter()
            .map(|(seconds, bid)| Quote {
                at: i.entered_at + Duration::seconds(seconds),
                bid,
                ask: bid + 0.1,
                gap_before: seconds == 90,
            })
            .collect();
        let mut whole = ContinuousReplay::new(i.clone(), "frozen-exit-v1").unwrap();
        whole.push_partition(&quotes).unwrap();
        let mut split = ContinuousReplay::new(i.clone(), "frozen-exit-v1").unwrap();
        split.push_partition(&quotes[..1]).unwrap();
        let cp = split.checkpoint();
        let encoded = serde_json::to_vec(&cp).unwrap();
        let decoded: Checkpoint = serde_json::from_slice(&encoded).unwrap();
        let mut restored =
            ContinuousReplay::restore(i.clone(), "frozen-exit-v1", &decoded, &quotes[..1]).unwrap();
        restored.push_partition(&quotes[1..]).unwrap();
        assert_eq!(whole.checkpoint(), restored.checkpoint());
        let coverage = Coverage::recorded_path(quotes[1].at);
        let a = whole.finish(coverage.clone(), Costs::default());
        let b = restored.finish(coverage, Costs::default());
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        assert_eq!(b.candidate.fills[0].price, 89.);
        assert!(b.candidate.net_r.is_none());
        assert!(ContinuousReplay::restore(i.clone(), "changed-engine", &cp, &quotes[..1]).is_err());
        let mut changed = i;
        changed.initial_r = 11.;
        assert!(ContinuousReplay::restore(changed, "frozen-exit-v1", &cp, &quotes[..1]).is_err());
    }
}
