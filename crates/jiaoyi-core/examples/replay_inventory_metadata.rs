//! Offline inventory adapter: expose the existing strategy calendar, never invent a broker calendar.
use chrono::{Duration, NaiveDate};
use jiaoyi_core::{strategy3::sessions, universe::AssetClass};
use serde::Deserialize;
use serde_json::json;
use std::io::{self, Read};

#[derive(Deserialize)]
struct Item {
    key: String,
    symbol: String,
    class: String,
    date: NaiveDate,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().take(4_000_001).read_to_string(&mut input)?;
    if input.len() > 4_000_000 {
        return Err("inventory input budget".into());
    }
    let items: Vec<Item> = serde_json::from_str(&input)?;
    let mut result = serde_json::Map::new();
    for item in items {
        let rule = AssetClass::parse(&item.class)
            .and_then(|class| sessions::rule_for(&item.symbol, class));
        let profile = rule.as_ref().map(|rule| match rule {
            sessions::SessionRule::Exchange { exchange, tz, open, close, lunch } => json!({
                "kind":"exchange_strategy", "exchange":exchange, "iana_timezone":tz.name(),
                "open":open.to_string(), "close":close.to_string(), "lunch":lunch,
                "weekdays_only":true}),
            sessions::SessionRule::UtcWindow { open, close } => json!({
                "kind":"fixed_utc_strategy", "iana_timezone":"UTC", "open":open.to_string(),
                "close":close.to_string(), "weekdays_only":true}),
            sessions::SessionRule::UtcDay => json!({"kind":"utc_day_strategy",
                "iana_timezone":"UTC", "open":"00:00:00", "close":"24:00:00", "weekdays_only":false}),
        });
        let session = rule
            .as_ref()
            .and_then(|r| sessions::session_on(r, item.date));
        let preparation: Vec<_> = (1..=45)
            .rev()
            .filter_map(|days| {
                rule.as_ref()
                    .and_then(|r| sessions::session_on(r, item.date - Duration::days(days)))
            })
            .collect();
        if result
            .insert(
                item.key,
                json!({"symbol":item.symbol,"profile":profile,
            "session":session,"prior_45_calendar_days":preparation,
            "authority":"verified_from_code", "holidays_included":false,
            "historical_broker_schedule_verified":false}),
            )
            .is_some()
        {
            return Err("duplicate inventory key".into());
        }
    }
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
