//! Eligibility at the saved original decision, never today's calendar.
use chrono::{DateTime, NaiveDate, Utc};
use jiaoyi_core::strategy3::{band, sessions::Session};
use serde_json::Value;

/// A complete saved session can establish a narrow entry-window check. It
/// does not reconstruct historical bands, first triggers or predictive edge.
pub(super) fn check(setup: &Value) -> Option<bool> {
    let (session, decided_at) = snapshot(setup)?;
    Some(band::entry_window(&session, decided_at).is_ok())
}

/// Separate from window eligibility: current S3 entry also requires a mark.
/// Call only after identifying the applicable registered mark rule.
pub(super) fn decision_mark_check(setup: &Value) -> Option<bool> {
    let (session, decided_at) = snapshot(setup)?;
    Some(band::at_decision_mark(&session, decided_at))
}

fn snapshot(setup: &Value) -> Option<(Session, DateTime<Utc>)> {
    let saved = setup.get("session")?;
    // Missing optional metadata is not proof that no lunch existed. Actual
    // Session serialization always writes either the tuple or explicit null.
    saved.get("lunch")?;
    let session: Session = serde_json::from_value(saved.clone()).ok()?;
    let declared_date: NaiveDate =
        serde_json::from_value(setup.get("session_date")?.clone()).ok()?;
    let decided_at: DateTime<Utc> =
        serde_json::from_value(setup.get("decided_at")?.clone()).ok()?;
    if session.date != declared_date
        || session.open >= session.close
        || session
            .lunch
            .is_some_and(|(start, end)| start < session.open || start >= end || end > session.close)
    {
        return None;
    }
    Some((session, decided_at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use serde_json::json;

    fn at(value: &str) -> DateTime<Utc> {
        value.parse().unwrap()
    }

    fn saved(decision: &str) -> Value {
        let session = Session {
            date: NaiveDate::from_ymd_opt(2026, 10, 27).unwrap(),
            open: at("2026-10-27T00:00:00Z"),
            close: at("2026-10-27T06:30:00Z"),
            lunch: Some((at("2026-10-27T02:30:00Z"), at("2026-10-27T03:30:00Z"))),
        };
        json!({"session":session,"session_date":session.date,"decided_at":decision})
    }

    #[test]
    fn saved_window_uses_exact_first_mark_lunch_and_last_thirty_minutes() {
        for (decision, expected) in [
            ("2026-10-26T23:59:00Z", false),
            ("2026-10-27T00:29:59.999Z", false),
            ("2026-10-27T00:30:00Z", true),
            ("2026-10-27T02:29:59.999Z", true),
            ("2026-10-27T02:30:00Z", false),
            ("2026-10-27T03:29:59.999Z", false),
            ("2026-10-27T03:30:00Z", true),
            ("2026-10-27T05:59:59.999Z", true),
            ("2026-10-27T06:00:00Z", false),
            ("2026-10-27T06:30:00Z", false),
        ] {
            assert_eq!(check(&saved(decision)), Some(expected), "{decision}");
        }
    }

    #[test]
    fn saved_decision_mark_is_separate_from_entry_window_eligibility() {
        for (decision, expected) in [
            ("2026-10-27T00:30:00Z", true),
            ("2026-10-27T00:30:04.999Z", true),
            ("2026-10-27T00:30:05Z", false),
            ("2026-10-27T00:59:59.999Z", false),
            ("2026-10-27T01:15:00Z", false),
        ] {
            let value = saved(decision);
            assert_eq!(check(&value), Some(true), "window {decision}");
            assert_eq!(
                decision_mark_check(&value),
                Some(expected),
                "mark {decision}"
            );
        }
        assert_eq!(
            decision_mark_check(&json!({"session_date":"2026-10-27"})),
            None
        );
    }

    #[test]
    fn legacy_missing_or_malformed_metadata_stays_unknown() {
        assert_eq!(check(&json!({"session_date":"2026-10-27"})), None);
        for field in ["session", "session_date", "decided_at"] {
            let mut value = saved("2026-10-27T01:00:00Z");
            value.as_object_mut().unwrap().remove(field);
            assert_eq!(check(&value), None, "missing {field}");
            value[field] = json!("not valid metadata");
            assert_eq!(check(&value), None, "malformed {field}");
        }
        let mut value = saved("2026-10-27T01:00:00Z");
        value["session"].as_object_mut().unwrap().remove("lunch");
        assert_eq!(check(&value), None);
        value["session"]["lunch"] = Value::Null;
        assert_eq!(
            check(&value),
            Some(true),
            "explicit no-lunch is measured metadata"
        );
        value["session"]["lunch"] = json!(["2026-10-27T02:00:00Z"]);
        assert_eq!(check(&value), None);
    }

    #[test]
    fn contradictory_dates_session_bounds_and_lunch_intervals_stay_unknown() {
        let mut value = saved("2026-10-27T01:00:00Z");
        value["session_date"] = json!("2026-10-28");
        assert_eq!(check(&value), None);
        for (open, close) in [
            ("2026-10-27T06:30:00Z", "2026-10-27T06:30:00Z"),
            ("2026-10-27T07:00:00Z", "2026-10-27T06:30:00Z"),
        ] {
            let mut value = saved("2026-10-27T01:00:00Z");
            value["session"]["open"] = json!(open);
            value["session"]["close"] = json!(close);
            assert_eq!(check(&value), None);
        }
        for lunch in [
            json!(["2026-10-27T03:30:00Z", "2026-10-27T03:30:00Z"]),
            json!(["2026-10-27T03:30:00Z", "2026-10-27T02:30:00Z"]),
            json!(["2026-10-26T23:59:00Z", "2026-10-27T02:30:00Z"]),
            json!(["2026-10-27T03:30:00Z", "2026-10-27T06:31:00Z"]),
        ] {
            let mut value = saved("2026-10-27T01:00:00Z");
            value["session"]["lunch"] = lunch;
            assert_eq!(check(&value), None);
        }
    }

    #[test]
    fn saved_exchange_trading_date_can_differ_from_utc_decision_date() {
        let session = Session {
            // ASX local28 October opens at23:00 UTC on27 October in DST.
            date: NaiveDate::from_ymd_opt(2026, 10, 28).unwrap(),
            open: at("2026-10-27T23:00:00Z"),
            close: at("2026-10-28T05:00:00Z"),
            lunch: None,
        };
        let value = json!({"session":session,"session_date":session.date,
            "decided_at":session.open+Duration::minutes(30)});
        assert_eq!(check(&value), Some(true));
        let restored: Session = serde_json::from_value(value["session"].clone()).unwrap();
        assert_eq!(
            restored, session,
            "shared serializer round-trip preserves original UTC bounds/date"
        );
    }
}
