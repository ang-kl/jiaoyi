//! Cache-only stock research, owner chat №17,618 §1 (28-10-2026 07:15 SGT).
//! No broker client, subscription, strategy, settings write or order capability.
//! A catalogue name match is evidence of a listing, not authenticated issuer identity.

use chrono::{DateTime, Duration, NaiveTime, Utc};
use jiaoyi_core::strategy3::sessions::{self, SessionRule};
use jiaoyi_store::{self as store, sqlx::PgPool};
use serde_json::{json, Value};

const METADATA_VERIFIED_AT: &str = "2026-10-28 07:49 SGT";
const CATALOGUE_MAX_AGE_HOURS: i64 = 24;

#[derive(Clone, Copy)]
struct Candidate {
    ticker: &'static str,
    issuer: &'static str,
    issuer_country: &'static str,
    exchange: &'static str,
    currency: &'static str,
    country_source: &'static str,
    exchange_source: &'static str,
    instrument: &'static str,
}

// A deliberately small, positively sourced basket, not a ticker/domain screener.
// Sources were inspected at METADATA_VERIFIED_AT. Never infer issuer country
// from a US listing or promote an unrecognised broker name into this basket.
const CANDIDATES: &[Candidate] = &[
    Candidate {
        ticker: "AAPL", issuer: "Apple Inc.", issuer_country: "US", exchange: "NASDAQ", currency: "USD",
        country_source: "https://investor.apple.com/investor-relations/faq/default.aspx",
        exchange_source: "https://investor.apple.com/investor-relations/faq/default.aspx", instrument: "ordinary_share",
    },
    Candidate {
        ticker: "MSFT", issuer: "Microsoft Corporation", issuer_country: "US", exchange: "NASDAQ", currency: "USD",
        country_source: "https://www.microsoft.com/en-us/investor/FAQ",
        exchange_source: "https://www.microsoft.com/en-us/investor/FAQ", instrument: "ordinary_share",
    },
    Candidate {
        ticker: "SAP", issuer: "SAP SE", issuer_country: "DE", exchange: "Xetra", currency: "EUR",
        country_source: "https://www.sap.com/swiss/about/legal/impressum.html",
        exchange_source: "https://www.sap.com/investors/en/resources/faq.html", instrument: "ordinary_share",
    },
    Candidate {
        ticker: "SIE", issuer: "Siemens AG", issuer_country: "DE", exchange: "Xetra", currency: "EUR",
        country_source: "https://www.siemens.com/en-gb/corporate-information/",
        exchange_source: "https://www.siemens.com/de-de/company/investor-relations/share-bonds-rating/basic-data-key-share-figures/", instrument: "ordinary_share",
    },
];

fn permitted(c: Candidate) -> bool {
    c.instrument == "ordinary_share"
        && !c.country_source.is_empty()
        && !c.exchange_source.is_empty()
        && matches!(
            (c.issuer_country, c.exchange),
            ("US", "NASDAQ") | ("DE", "Xetra")
        )
}

fn rule(exchange: &str) -> SessionRule {
    let time = |h, m| NaiveTime::from_hms_opt(h, m, 0).unwrap();
    let (tz, open, close) = match exchange {
        "NASDAQ" => (chrono_tz::America::New_York, time(9, 30), time(16, 0)),
        "Xetra" => (chrono_tz::Europe::Berlin, time(9, 0), time(17, 30)),
        _ => unreachable!("only embedded verified exchanges"),
    };
    SessionRule::Exchange {
        exchange: if exchange == "NASDAQ" {
            "NASDAQ"
        } else {
            "Xetra"
        },
        tz,
        open,
        close,
        lunch: None,
    }
}

fn sgt(at: DateTime<Utc>) -> String {
    at.with_timezone(&chrono_tz::Asia::Singapore)
        .format("%Y-%m-%d %H:%M:%S SGT")
        .to_string()
}

struct Catalogue {
    run: i64,
    at: DateTime<Utc>,
    list: Value,
    assets: Option<Value>,
}

/// Exact, account-local evidence only. Even a coherent name/asset match lacks
/// broker issuer/exchange identifiers in the current loader's stored schema.
fn broker(c: Candidate, catalogue: Option<&Catalogue>, now: DateTime<Utc>) -> Value {
    let Some(cat) = catalogue else {
        return json!({"state":"catalogue_missing"});
    };
    let Some(list) = cat.list.as_array() else {
        return json!({"state":"catalogue_invalid"});
    };
    let matches: Vec<_> = list
        .iter()
        .filter(|s| s["name"].as_str() == Some(c.ticker))
        .collect();
    let mut out = json!({"source_run_id":cat.run,"source_at":cat.at,"source_at_sgt":sgt(cat.at),
        "exact_name_matches":matches.len()});
    let state = if cat.at > now || now - cat.at > Duration::hours(CATALOGUE_MAX_AGE_HOURS) {
        "catalogue_stale_or_future"
    } else if matches.is_empty() {
        "not_listed"
    } else if matches.len() != 1 {
        "ambiguous"
    } else {
        let symbol = matches[0];
        // ID collisions are ambiguous even if the other name differs.
        match symbol["id"].as_i64().filter(|id| *id > 0) {
            None => "identity_missing",
            Some(id) if list.iter().filter(|s| s["id"].as_i64() == Some(id)).count() != 1 => {
                "ambiguous"
            }
            Some(id) => {
                out["symbol_id"] = json!(id);
                out["name"] = json!(c.ticker);
                out["enabled"] = json!(symbol["enabled"].as_bool());
                match symbol["enabled"].as_bool() {
                    Some(false) => "disabled",
                    None => "enabled_unknown",
                    Some(true) => {
                        let assets = cat.assets.as_ref().and_then(Value::as_array);
                        let asset = |field: &str| -> Option<&str> {
                            let id = symbol[field].as_i64()?;
                            let matches: Vec<_> = assets?
                                .iter()
                                .filter(|a| a["asset_id"].as_i64() == Some(id))
                                .collect();
                            if matches.len() == 1 {
                                matches[0]["name"].as_str()
                            } else {
                                None
                            }
                        };
                        match (asset("base_asset_id"), asset("quote_asset_id")) {
                            (Some(base), Some(quote))
                                if base == c.ticker && quote == c.currency =>
                            {
                                "matched_name_identity_unverified"
                            }
                            (Some(_), Some(_)) => "asset_identity_mismatch",
                            _ => "asset_identity_missing",
                        }
                    }
                }
            }
        }
    };
    out["state"] = json!(state);
    out
}

fn account_view(account: i64, catalogue: Option<&Catalogue>, now: DateTime<Utc>) -> Value {
    let mut receipts = Vec::new();
    for exchange in ["NASDAQ", "Xetra"] {
        let rule = rule(exchange);
        // One current receipt per exchange/session. Outside a session, show
        // the next weekday; never label today's missed deadline as tomorrow's.
        let session = sessions::session_at(&rule, now)
            .or_else(|| sessions::next_session(&rule, now))
            .unwrap();
        let deadline = session.open - Duration::minutes(crate::strategy3::preopen::LEAD_MINUTES);
        let starts = session.open - Duration::minutes(crate::strategy3::preopen::START_MINUTES);
        let candidates: Vec<_> = CANDIDATES.iter().copied().filter(|c| c.exchange == exchange && permitted(*c)).map(|c| {
            json!({"ticker":c.ticker,"issuer":c.issuer,"issuer_country":c.issuer_country,
                "exchange":c.exchange,"currency":c.currency,"asset_class":"stock","instrument":c.instrument,
                "metadata_verified_at_sgt":METADATA_VERIFIED_AT,
                "country_source":c.country_source,"exchange_source":c.exchange_source,
                "broker":broker(c,catalogue,now),"eligible":false,
                "missing_inputs":["broker_issuer_exchange_identity","stock_quote_history_collection","verified_cash_calendar"]})
        }).collect();
        receipts.push(json!({"exchange":exchange,"clock":rule.describe(),"session_open":session.open,
            "session_open_sgt":sgt(session.open),"session_close":session.close,
            "preparation_starts_at":starts,"preparation_starts_at_sgt":sgt(starts),
            "deadline_at":deadline,"deadline_at_sgt":sgt(deadline),
            "deadline_state":if now > deadline {"missed"} else if now >= starts {"due"} else {"scheduled"},
            "calendar_state":"weekdays_only_holidays_unverified",
            "preparation_state":"unavailable","prepared_at":null,"deadline_met":false,
            "candidates":candidates}));
    }
    json!({"account_id":account,"sessions":receipts})
}

/// One compact current projection per session; GET does not append snapshots
/// or manufacture an earlier completion time. No quote archive or new polling.
pub async fn report(pool: &PgPool, accounts: &[i64], now: DateTime<Utc>) -> store::Result<Value> {
    let mut ids = accounts.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let mut rows = Vec::new();
    for account in ids {
        let cat = match store::latest_listed_symbols(pool, account).await? {
            Some((run, at, list)) => {
                // Fetch the SAME run, never combine a newer symbol list with
                // an older successful asset read from latest_assets.
                let assets = store::assets_of_run(pool, account, run)
                    .await?
                    .map(|(assets, _)| assets);
                Some(Catalogue {
                    run,
                    at,
                    list,
                    assets,
                })
            }
            None => None,
        };
        rows.push(account_view(account, cat.as_ref(), now));
    }
    Ok(
        json!({"source_chat":"№17,618 §1","source_at_sgt":"2026-10-28 07:15 SGT",
        "parent_chat":"№17,616 §1","parent_source_at_sgt":"2026-10-28 07:09 SGT",
        "generated_at":now,"generated_at_sgt":sgt(now),"stock_trading_active":false,
        "scope":"read-only research watchlist; stock preparation unavailable; no registered stock strategy",
        "preparation_limit":"cached listings lack issuer/exchange authentication; existing collection covers the live universe only",
        "catalogue_max_age_hours":CATALOGUE_MAX_AGE_HOURS,
        "liquidity":"large-cap candidates; current broker liquidity unverified",
        "financial_movement":"not measurable",
        "goals":"per strategy/account latest20 ≥73% net wins; realized payoff objective 1:1.38; planned geometry separate",
        "accounts":rows}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        "2026-10-28T07:40:00Z".parse().unwrap()
    }
    fn catalogue() -> Catalogue {
        Catalogue {
            run: 7,
            at: now(),
            list: json!([{"id":123,"name":"AAPL","enabled":true,"base_asset_id":1,"quote_asset_id":2}]),
            assets: Some(json!([{"asset_id":1,"name":"AAPL"},{"asset_id":2,"name":"USD"}])),
        }
    }

    #[test]
    fn positive_issuer_evidence_includes_us_europe_but_excludes_china_adrs_and_indices() {
        assert!(CANDIDATES.iter().copied().all(permitted));
        assert!(CANDIDATES.iter().any(|c| c.issuer_country == "US"));
        assert!(CANDIDATES.iter().any(|c| c.issuer_country == "DE"));
        for country in ["CN", "HK", "KY", "", "unknown"] {
            assert!(!permitted(Candidate {
                issuer_country: country,
                ..CANDIDATES[0]
            }));
        }
        assert!(!permitted(Candidate {
            instrument: "index",
            ..CANDIDATES[0]
        }));
        assert!(!permitted(Candidate {
            instrument: "adr",
            issuer_country: "CN",
            ..CANDIDATES[0]
        }));
        assert!(!permitted(Candidate {
            country_source: "",
            ..CANDIDATES[0]
        }));
    }

    #[test]
    fn exact_broker_identity_refuses_missing_ambiguous_disabled_stale_and_wrong_currency() {
        let state = |c: &Catalogue| {
            broker(CANDIDATES[0], Some(c), now())["state"]
                .as_str()
                .unwrap()
                .to_owned()
        };
        let mut c = catalogue();
        assert_eq!(state(&c), "matched_name_identity_unverified");
        c.list[0]["enabled"] = json!(false);
        assert_eq!(state(&c), "disabled");
        c.list[0]["enabled"] = Value::Null;
        assert_eq!(state(&c), "enabled_unknown");
        c = catalogue();
        c.list[0]["name"] = json!("AAPL.US");
        assert_eq!(state(&c), "not_listed");
        c = catalogue();
        let duplicate = c.list[0].clone();
        c.list.as_array_mut().unwrap().push(duplicate);
        assert_eq!(state(&c), "ambiguous");
        c = catalogue();
        c.list
            .as_array_mut()
            .unwrap()
            .push(json!({"id":123,"name":"OTHER"}));
        assert_eq!(state(&c), "ambiguous");
        c = catalogue();
        c.at -= Duration::hours(25);
        assert_eq!(state(&c), "catalogue_stale_or_future");
        c.at = now() + Duration::seconds(1);
        assert_eq!(state(&c), "catalogue_stale_or_future");
        c = catalogue();
        c.assets.as_mut().unwrap()[1]["name"] = json!("EUR");
        assert_eq!(state(&c), "asset_identity_mismatch");
        c.assets = None;
        assert_eq!(state(&c), "asset_identity_missing");
        c = catalogue();
        c.list[0]["id"] = Value::Null;
        assert_eq!(state(&c), "identity_missing");
    }

    #[test]
    fn stock_clocks_follow_dst_weekends_and_never_certify_catalogue_as_preparation() {
        for (exchange, at, expected) in [
            ("NASDAQ", "2026-10-28T12:00:00Z", "2026-10-28T13:30:00Z"),
            ("NASDAQ", "2026-11-02T12:00:00Z", "2026-11-02T14:30:00Z"),
            ("Xetra", "2026-10-23T05:00:00Z", "2026-10-23T07:00:00Z"),
            ("Xetra", "2026-10-25T12:00:00Z", "2026-10-26T08:00:00Z"),
        ] {
            let s = sessions::next_session(&rule(exchange), at.parse().unwrap()).unwrap();
            assert_eq!(s.open, expected.parse::<DateTime<Utc>>().unwrap());
            let v = account_view(1, Some(&catalogue()), s.open - Duration::minutes(10));
            let receipt = v["sessions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["exchange"] == exchange)
                .unwrap();
            assert_eq!(
                receipt["deadline_at"],
                json!(s.open - Duration::minutes(10))
            );
            assert_eq!(receipt["deadline_met"], false);
            assert!(receipt["prepared_at"].is_null());
            assert_eq!(receipt["preparation_state"], "unavailable");
        }
        let v = account_view(1, None, "2026-10-28T13:30:00Z".parse().unwrap());
        assert_eq!(v["sessions"][0]["deadline_state"], "missed");
        assert_eq!(v["sessions"][0]["candidates"][0]["eligible"], false);
    }
}
