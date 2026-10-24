//! The held-out seal of the pre-registration (docs/preregistration-2026-10-24.md,
//! approved by the owner 24-10-2026): the held-out period is 2025-10-24 →
//! 2026-10-23, the development period everything before. Development work
//! never reads a price dated on or after the seal; a request that reaches
//! into the sealed period is refused, never clamped.

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};

/// The first day of the held-out period.
pub fn held_out_from() -> NaiveDate {
    NaiveDate::from_ymd_opt(2025, 10, 24).expect("valid date")
}

/// The last day of the held-out period.
pub fn held_out_to() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 23).expect("valid date")
}

/// The seal as an instant: 2025-10-24 00:00 UTC.
pub fn seal_instant() -> DateTime<Utc> {
    Utc.from_utc_datetime(&held_out_from().and_hms_opt(0, 0, 0).expect("midnight"))
}

/// The end of a daily bar: 24 hours after its open. A bar dated by the UTC
/// date of its open can run into the next day (FX daily bars open at 21:00 or
/// 22:00 UTC), so a bar is development data only when it ENDS by the seal.
pub fn daily_bar_end(open: DateTime<Utc>) -> DateTime<Utc> {
    open + Duration::days(1)
}

/// Does a daily bar opening at `open` hold only development-period prices?
pub fn daily_bar_is_development(open: DateTime<Utc>) -> bool {
    daily_bar_end(open) <= seal_instant()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bar_is_development_only_when_it_ends_by_the_seal() {
        let at = |s: &str| s.parse::<DateTime<Utc>>().unwrap();
        // Opens at midnight on 23-10: ends exactly at the seal.
        assert!(daily_bar_is_development(at("2025-10-23T00:00:00Z")));
        // Opens 21:00 on 22-10: ends 21:00 on 23-10.
        assert!(daily_bar_is_development(at("2025-10-22T21:00:00Z")));
        // Opens 21:00 on 23-10 (dated 23-10): runs to 21:00 on 24-10, sealed.
        assert!(!daily_bar_is_development(at("2025-10-23T21:00:00Z")));
        assert!(!daily_bar_is_development(at("2025-10-24T00:00:00Z")));
        assert_eq!(seal_instant(), at("2025-10-24T00:00:00Z"));
        assert_eq!(
            held_out_to(),
            NaiveDate::from_ymd_opt(2026, 10, 23).unwrap()
        );
    }
}
