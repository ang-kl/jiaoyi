//! Account-wide FX preparation runs independently from entry/exit handlers.
//! It only reads broker state, subscribes conversion quotes and records changes.
use super::*;

pub(super) async fn run(
    pool: &PgPool,
    session: &AccountSession<'_>,
    mut shutdown: Shutdown,
) -> anyhow::Result<()> {
    let account = session.account().id();
    let conn = session.connection();
    let mut timer = tokio::time::interval(Duration::from_secs(2));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut last_revision = None;
    let mut last_attempt = tokio::time::Instant::now() - Duration::from_secs(30);
    let mut last_error = None;
    let mut decisions = Decisions::new(pool, account);
    loop {
        tokio::select! {
            _ = shutdown.wait() => return Ok(()),
            _ = timer.tick() => {}
        }
        let revision = conn.entry_revision(account);
        if last_revision != Some(revision)
            || last_attempt.elapsed()
                >= Duration::from_secs(if last_error.is_some() { 6 } else { 30 })
        {
            last_attempt = tokio::time::Instant::now();
            let warm = async {
                let (trader, book) = tokio::join!(session.trader(), session.margin_book());
                jiaoyi_ctrader::margin::warm_account_conversions(
                    session,
                    &trader.map_err(|e| e.to_string())?,
                    &book.map_err(|e| e.to_string())?,
                )
                .await?;
                hydrate_closed_book(pool, session).await
            };
            let result = tokio::select! {
                _ = shutdown.wait() => return Ok(()),
                result = warm => result,
            };
            last_error = result.err();
            last_revision = Some(revision);
        }
        let mut status = conn.conversion_status(account);
        status["warmup_error"] = json!(last_error);
        status["closed_book_policy"] = json!(jiaoyi_ctrader::margin::CLOSED_BOOK_POLICY);
        status["closed_book_quotes"] = json!(jiaoyi_ctrader::margin::closed_book_quotes(session));
        // Prices/timestamps change every quote; persist only path/availability
        // changes, with the contemporaneous full provenance in the detail.
        let states: Vec<_> = status["paths"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| {
                json!({"from_asset":p["from_asset"],"deposit_asset":p["deposit_asset"],
                "legs":p["legs"],"state":p["state"],"subscribed":p["subscribed"]})
            })
            .collect();
        let reason = json!({"paths":states,"closed_book_symbols":jiaoyi_ctrader::margin::closed_book_quotes(session).iter().map(|q|q.calendar.symbol_id).collect::<Vec<_>>(),"warmup_error":last_error}).to_string();
        decisions.note(None, "exposure_fx", &reason, status).await;
    }
}

pub(super) async fn health(pool: &PgPool, account: i64) -> Value {
    match ts::latest_decision_detail(pool, account, "s3_exposure_fx").await {
        Ok(Some((at, detail))) => json!({"last_state_change_at":at,"recorded_provenance":detail,
            "freshness_policy":"Candidate FX stays live (5s). Existing positions may use verified scheduled-closure pre-close evidence with a 5% adverse reserve; the writer revalidates the closure. This record is not an executable quote."}),
        Ok(None) => json!({"state":"not_observed"}),
        Err(_) => json!({"state":"unavailable"}),
    }
}

async fn hydrate_closed_book(pool: &PgPool, session: &AccountSession<'_>) -> Result<(), String> {
    use jiaoyi_ctrader::{margin, market_hours::ContractCalendar};
    let account = session.account().id();
    let revision = session.connection().entry_revision(account);
    let now = Utc::now();
    let status = session.connection().conversion_status(account);
    let ids: std::collections::BTreeSet<i64> = status["paths"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|p| p["legs"].as_array().into_iter().flatten())
        .filter_map(|l| l["symbol_id"].as_i64())
        .collect();
    let cached = margin::closed_book_quotes(session);
    let need: Vec<i64> = ids
        .into_iter()
        .filter(|id| {
            !cached
                .iter()
                .any(|q| q.calendar.symbol_id == *id && (now - q.verified_at).num_seconds() < 240)
        })
        .collect();
    if need.is_empty() {
        return Ok(());
    }
    let contracts = session
        .symbols_by_id(&need)
        .await
        .map_err(|e| e.to_string())?;
    let mut errors = Vec::new();
    for id in need {
        let result = async {
        let matches: Vec<_> = contracts
            .symbol
            .iter()
            .filter(|s| s.symbol_id == id)
            .collect();
        if matches.len() != 1 {
            return Err("closed-book conversion calendar identity missing/ambiguous".into());
        }
        let calendar = ContractCalendar::from_symbol(matches[0]).map_err(|e| e.to_string())?;
        let Some((close, _)) = calendar.closed_window(now) else {
            return Ok(());
        };
        let memory = session
            .connection()
            .broker_dated_quote_snapshot(account, id)
            .and_then(|q| {
                let (b, bt) = q.bid?;
                let (a, at) = q.ask?;
                Some(margin::ClosedBookQuote {
                    account_id: account,
                    calendar: calendar.clone(),
                    bid: b,
                    ask: a,
                    bid_at: bt,
                    ask_at: at,
                    verified_at: now,
                    source: "connection_broker_timestamp_preclose_quote".into(),
                })
            })
            .filter(|q| q.validate(account, now).is_ok());
        let evidence = match memory {
            Some(q) => q,
            None => {
                let q=jiaoyi_store::ticks::preclose_quote(pool,account,id,close).await.map_err(|e|e.to_string())?
                    .ok_or_else(||format!("closed-book FX pre-close evidence missing: account {account}, symbol {id}, close {close}"))?;
                margin::ClosedBookQuote {
                    account_id: account,
                    calendar,
                    bid: q.bid,
                    ask: q.ask,
                    bid_at: q.bid_at,
                    ask_at: q.ask_at,
                    verified_at: now,
                    source: q.source,
                }
            }
        };
        if session.connection().entry_revision(account) != revision {
            return Err("account changed during closed-book hydration".into());
        }
        margin::install_closed_book_quote(session, evidence)
        }.await;
        if let Err(error) = result {
            errors.push(format!("symbol {id}: {error}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};
    use jiaoyi_core::secret::Secret;
    use jiaoyi_ctrader::{
        margin, messages as pb,
        testkit::FakeBroker,
        trading::{Command, Side, VolumeLimits},
    };
    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn cold_start_hydrates_only_existing_book_from_account_scoped_preclose_ticks(
        pool: PgPool,
    ) {
        let account = 49236051;
        jiaoyi_store::upsert_account(&pool, account, Environment::Demo)
            .await
            .unwrap();
        let now = Utc::now();
        let second = now.weekday().num_days_from_sunday() * 86400 + now.num_seconds_from_midnight();
        let start = second.saturating_sub(60);
        let end = (second + 60).min(604800);
        let mut intervals = vec![];
        if start > 0 {
            intervals.push(pb::ProtoOaInterval {
                start_second: 0,
                end_second: start,
            });
        }
        if end < 604800 {
            intervals.push(pb::ProtoOaInterval {
                start_second: end,
                end_second: 604800,
            });
        }
        let contract = |id| pb::ProtoOaSymbol {
            symbol_id: id,
            schedule_time_zone: Some("UTC".into()),
            schedule: intervals.clone(),
            ..Default::default()
        };
        let close = jiaoyi_ctrader::market_hours::ContractCalendar::from_symbol(&contract(5))
            .unwrap()
            .closed_window(now)
            .unwrap()
            .0;
        let quote_at = close - chrono::Duration::seconds(1);
        jiaoyi_store::ticks::ensure_partitions(&pool, quote_at.date_naive(), 1)
            .await
            .unwrap();
        let mut rows = vec![];
        for (id, b, a) in [(5, 65000, 66000), (1, 110000, 111000)] {
            rows.push(jiaoyi_store::ticks::TickRow {
                account_id: account,
                session_id: "preclose".into(),
                seq: id,
                broker_symbol_id: id,
                received_at: quote_at,
                broker_timestamp: Some(quote_at.timestamp_millis()),
                bid_e5: Some(b),
                ask_e5: Some(a),
            });
        }
        jiaoyi_store::ticks::insert(&pool, &rows[..1])
            .await
            .unwrap();
        assert!(
            jiaoyi_store::ticks::preclose_quote(&pool, account + 1, 5, close)
                .await
                .unwrap()
                .is_none()
        );
        let light = |id, base, quote| pb::ProtoOaLightSymbol {
            symbol_id: id,
            base_asset_id: Some(base),
            quote_asset_id: Some(quote),
            ..Default::default()
        };
        let trader = pb::ProtoOaTrader {
            ctid_trader_account_id: account,
            balance: 1_000_000,
            deposit_asset_id: 15,
            money_digits: Some(2),
            ..Default::default()
        };
        let fake = FakeBroker {
            account_id: account,
            trader: Some(trader.clone()),
            symbols: vec![contract(5), contract(1)],
            light_symbols: vec![
                light(10, 99, 2),
                light(11, 98, 3),
                light(41, 97, 15),
                light(22, 96, 15),
                light(5, 2, 15),
                light(1, 3, 15),
            ],
            ..Default::default()
        };
        for (id, symbol, entry, stop, volume) in [
            (101, 10, 1000.0, 900.0, 100),
            (102, 11, 2000.0, 1900.0, 100),
            (103, 41, 3878.12, 3893.62, 1000),
        ] {
            fake.positions.lock().unwrap().push(pb::ProtoOaPosition {
                position_id: id,
                position_status: 1,
                price: Some(entry),
                stop_loss: Some(stop),
                used_margin: Some(100),
                money_digits: Some(2),
                trade_data: pb::ProtoOaTradeData {
                    symbol_id: symbol,
                    volume,
                    trade_side: if symbol == 41 { 2 } else { 1 },
                    label: Some(
                        if symbol == 41 {
                            "jiaoyi-s3"
                        } else {
                            "jiaoyi-s1"
                        }
                        .into(),
                    ),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
        let (conn, log) = fake.spawn();
        conn.application_auth("id", &Secret::new("secret"))
            .await
            .unwrap();
        let allow = Allowlist::parse(Some("49236051")).unwrap();
        let session = conn
            .account_auth(&allow.permit(account).unwrap(), &Secret::new("token"))
            .await
            .unwrap();
        let book = session.margin_book().await.unwrap();
        margin::warm_account_conversions(&session, &trader, &book)
            .await
            .unwrap();
        assert!(conn.quote_snapshot(account, 5).is_none());
        assert!(hydrate_closed_book(&pool, &session)
            .await
            .unwrap_err()
            .contains("pre-close evidence missing"));
        assert_eq!(
            margin::closed_book_quotes(&session)
                .iter()
                .map(|q| q.calendar.symbol_id)
                .collect::<Vec<_>>(),
            vec![5],
            "one missing currency cannot prevent independent valid evidence from hydrating"
        );
        jiaoyi_store::ticks::insert(&pool, &rows[1..])
            .await
            .unwrap();
        hydrate_closed_book(&pool, &session).await.unwrap();
        assert!(
            conn.quote_snapshot(account, 5).is_none(),
            "hydration must not populate executable quotes"
        );
        let command = Command::Open {
            symbol_id: 22,
            side: Side::Buy,
            volume: 1,
            relative_stop: 100_000,
            relative_take_profit: None,
            limits: VolumeLimits {
                min: 1,
                step: 1,
                max: None,
                digits: 2,
            },
            quote_to_deposit: Some(1.0),
            label: "jiaoyi-s3".into(),
        };
        let receipt = margin::check(&session, &trader, &book, &command, 250.0)
            .await
            .unwrap();
        let proof = receipt.fx_guard.unwrap();
        assert!((proof.base_loss_reserve - 155.01).abs() < 1e-7);
        assert!(
            (margin::existing_book_reserve(
                account,
                &proof.existing_book_exposures,
                &proof.closed_book_quotes,
                &proof.conversion_calendars,
                Utc::now(),
                |_| None
            )
            .unwrap()
                - (66.0 + 111.0) * 1.05)
                .abs()
                < 1e-7
        );
        let mut no_stop = book.clone();
        no_stop.position[0].stop_loss = None;
        assert!(margin::check(&session, &trader, &no_stop, &command, 250.0)
            .await
            .unwrap_err()
            .contains("no verified stop"));
        assert!(
            log.lock().unwrap().orders.is_empty(),
            "read-only reserve verification sends no trades"
        );
    }
}
