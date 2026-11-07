//! Offline production-rule oil scenario. Evidence is supplied, never invented.
use super::{s3_signal::Request, s3_ticks};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Preparation {
    /// Evidence reference, e.g. immutable preparation receipt/file hash.
    pub source: String,
    pub available_at: DateTime<Utc>,
    pub opening_bar_tick_volume: i64,
    pub spread: f64,
    pub commission_round_trip: f64,
}

pub fn study(
    req: &Request,
    src: &s3_ticks::Source,
    evidence: &Preparation,
) -> Result<s3_ticks::Study, String> {
    req.validate(Utc::now())?;
    if !matches!(req.symbol.as_str(), "SpotCrude" | "SpotBrent")
        || req
            .tick_replay
            .as_ref()
            .is_none_or(|o| !o.entry_rule.is_current())
    {
        return Err("prepared oil replay requires SpotCrude/SpotBrent current S3".into());
    }
    if evidence.source.trim().is_empty()
        || evidence.opening_bar_tick_volume <= 0
        || evidence.available_at < req.from
        || evidence.available_at > req.from + Duration::minutes(30)
        || !evidence.spread.is_finite()
        || !evidence.commission_round_trip.is_finite()
        || evidence.spread != req.spread
        || evidence.commission_round_trip != req.commission_round_trip
    {
        return Err("missing, late or inconsistent opening/prepared-cost evidence".into());
    }
    // Validate the complete supplied source before slicing; malformed old bars
    // must not disappear silently from validation.
    super::s3_signal::study(req, &src.bars)?;
    let mut causal = src.clone();
    causal
        .bars
        .retain(|b| b.at >= req.from - Duration::days(45));
    let mut result = s3_ticks::study_impl(req, &causal, true)?;
    result.mode = "prepared_oil_s3_v7_scenario_v1";
    result.notes.push("Preparation reference and opening volume are supplied evidence, not independently authenticated broker receipts. 45-day production history view; final cost is rechecked at the next executable quote, used as a dispatch proxy. Account admission, transport latency and broker acknowledgement remain unmodelled; not financial qualification.");
    Ok(result)
}
