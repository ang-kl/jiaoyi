//! Bounded, deterministic health episodes; consumption/notifications belong to
//! the existing engineering incident pipeline, not the one-second evaluator.
use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const RECOVERY_MAX_AGE_SECS: i64 = 15;
const HEALTHY_OBSERVATIONS: u8 = 6;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Incident {
    pub id: String,
    pub kind: String,
    pub state: String,
    pub episode: u64,
    pub opened_at: DateTime<Utc>,
    pub last_observed_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub reason: String,
    pub evidence_revision: u64,
    pub occurrences: u64,
    pub next_check_at: DateTime<Utc>,
    #[serde(skip)]
    healthy_count: u8,
    #[serde(skip)]
    last_healthy_at: Option<DateTime<Utc>>,
}

#[derive(Default)]
pub struct Incidents(BTreeMap<String, Incident>);

fn reason(kind: &str) -> Option<&'static str> {
    match kind {
        "deadline_missed" => Some("assessment_completion_deadline_missed"),
        "source_unavailable" => Some("broker_source_unavailable"),
        "disconnected" => Some("broker_session_disconnected"),
        _ => None,
    }
}

impl Incidents {
    pub fn rows(&self) -> Vec<&Incident> {
        self.0.values().collect()
    }

    /// Restore episode counters, including resolved episodes, never broker data.
    /// The timestamp floor prevents a new process reusing an old episode while
    /// a delayed database read is still in flight.
    pub fn restore(&mut self, account: i64, value: &Value) {
        for item in value.as_array().into_iter().flatten().take(3) {
            let Ok(prior) = serde_json::from_value::<Incident>(item.clone()) else {
                continue;
            };
            if reason(&prior.kind) != Some(prior.reason.as_str())
                || prior.id != format!("setup-assessment:{account}:{}", prior.kind)
                || !matches!(prior.state.as_str(), "active" | "resolved")
                || prior.episode == 0
                || prior.last_observed_at < prior.opened_at
                || (prior.state == "resolved"
                    && prior.resolved_at.is_none_or(|t| t < prior.opened_at))
            {
                continue;
            }
            if self
                .0
                .get(&prior.kind)
                .is_none_or(|current| current.episode < prior.episode)
            {
                self.0.insert(prior.kind.clone(), prior);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn observe(
        &mut self,
        account: i64,
        at: DateTime<Utc>,
        revision: u64,
        healthy: bool,
        deadline_missed: bool,
        source_failed: bool,
        connected: bool,
    ) {
        for (kind, bad) in [
            ("deadline_missed", deadline_missed),
            ("source_unavailable", source_failed),
            ("disconnected", !connected),
        ] {
            if bad {
                let existing = self.0.get(kind);
                if existing.is_none_or(|i| i.state == "resolved") {
                    let episode = existing
                        .map_or(1, |i| i.episode.saturating_add(1))
                        .max(at.timestamp_millis().max(1) as u64);
                    self.0.insert(
                        kind.into(),
                        Incident {
                            id: format!("setup-assessment:{account}:{kind}"),
                            kind: kind.into(),
                            state: "active".into(),
                            episode,
                            opened_at: at,
                            last_observed_at: at,
                            resolved_at: None,
                            reason: reason(kind).unwrap().into(),
                            evidence_revision: revision,
                            occurrences: 0,
                            next_check_at: at + Duration::seconds(1),
                            healthy_count: 0,
                            last_healthy_at: None,
                        },
                    );
                }
            }
            let Some(i) = self.0.get_mut(kind) else {
                continue;
            };
            i.last_observed_at = at;
            i.evidence_revision = revision;
            i.next_check_at = at + Duration::seconds(1);
            if bad {
                i.occurrences = i.occurrences.saturating_add(1);
                i.healthy_count = 0;
                i.last_healthy_at = None;
            } else if !healthy || deadline_missed {
                i.healthy_count = 0;
                i.last_healthy_at = None;
            } else if i.state == "active"
                && i.last_healthy_at
                    .is_none_or(|last| (at - last).num_milliseconds() >= 1000)
            {
                i.healthy_count = i.healthy_count.saturating_add(1);
                i.last_healthy_at = Some(at);
                if i.healthy_count >= HEALTHY_OBSERVATIONS {
                    i.state = "resolved".into();
                    i.resolved_at = Some(at);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn repeated_failures_are_one_bounded_episode_and_recovery_requires_fresh_completed_checks() {
        let t = Utc::now();
        let mut h = Incidents::default();
        h.observe(7, t, 1, false, true, true, false);
        let first = serde_json::to_value(h.rows()).unwrap();
        assert_eq!(h.rows().len(), 3);
        for n in 1..100 {
            h.observe(
                7,
                t + Duration::seconds(n),
                n as u64,
                false,
                true,
                true,
                false,
            );
        }
        assert_eq!(h.rows().len(), 3);
        assert_eq!(h.rows()[0].episode, first[0]["episode"].as_u64().unwrap());
        // Fresh timer alone and fast event bursts cannot resolve an incident.
        for n in 100..110 {
            h.observe(
                7,
                t + Duration::seconds(n),
                n as u64,
                false,
                false,
                false,
                true,
            );
        }
        for n in 0..20 {
            h.observe(
                7,
                t + Duration::seconds(110) + Duration::milliseconds(n),
                110,
                true,
                false,
                false,
                true,
            );
        }
        assert!(h.rows().iter().all(|i| i.state == "active"));
        for n in 111..116 {
            h.observe(
                7,
                t + Duration::seconds(n),
                n as u64,
                true,
                false,
                false,
                true,
            );
        }
        assert!(h.rows().iter().all(|i| i.state == "resolved"));
        let saved = serde_json::to_value(h.rows()).unwrap();
        let mut restarted = Incidents::default();
        restarted.restore(7, &saved);
        restarted.observe(7, t + Duration::seconds(116), 0, false, true, false, true);
        assert!(restarted.0["deadline_missed"].episode > h.0["deadline_missed"].episode);
        assert_eq!(
            restarted.0["source_unavailable"].episode,
            h.0["source_unavailable"].episode
        );
        // Delayed restoration must not replace a newer occurrence.
        restarted.restore(7, &saved);
        assert_eq!(restarted.0["deadline_missed"].state, "active");
        let before = restarted.rows().len();
        restarted.restore(8, &saved);
        restarted.restore(7, &json!([{"kind":"unknown"}]));
        assert_eq!(restarted.rows().len(), before);
    }
}
