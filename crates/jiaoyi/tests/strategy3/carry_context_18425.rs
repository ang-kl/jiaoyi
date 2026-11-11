use super::*;
use jiaoyi_core::strategy3::{macro_pullback as mp, production_sessions as prod};

fn current_preparation(day: &strategy3::Day, session: Session) -> strategy3::preopen::Prepared {
    strategy3::preopen::Prepared {
        symbol: "BTCUSD".into(),
        symbol_id: Some(BTC),
        session,
        prereg_hash: strategy3::prereg_hash(),
        source_cutoff_at: session.open - Duration::minutes(20),
        completed_at: session.open - Duration::minutes(19),
        table: Some(day.table.clone()),
        macro_context: day.macro_context.as_ref().ok().copied(),
        macro_context_error: None,
        spread_ready: true,
        commission_ready: true,
        metadata_ready: true,
        missing_marks: 0,
        source_error: false,
    }
}

/// Exercise the real preparation and durable arm paths. The current opening
/// differs materially from the old arm origin so accidental coupling fails.
#[sqlx::test(migrator = "store::MIGRATOR")]
async fn carried_source_survives_current_peer_rollover_and_both_receipt_formats(pool: PgPool) {
    loaded(&pool).await;
    let fixture = fake((60_455.0, 60_462.0));
    let current = prod::session_at(prod::Market::Crypto, at(250)).unwrap();
    let current_open = OPEN + 500.0;
    store::ticks::insert(
        &pool,
        &[store::ticks::TickRow {
            account_id: DEMO,
            session_id: "18425-current-opening".into(),
            seq: 0,
            broker_symbol_id: BTC,
            received_at: current.open,
            broker_timestamp: Some(current.open.timestamp_millis()),
            bid_e5: Some(e5(current_open - 2.5)),
            ask_e5: Some(e5(current_open + 2.5)),
        }],
    )
    .await
    .unwrap();
    with_trader!(&pool, fixture, Environment::Demo, |t, log| {
        t.on_quote(BTC, Some(OPEN + 897.5), Some(OPEN + 902.5), at(210))
            .await
            .unwrap();
        let original = t.syms[&BTC].pullback.expect("late-window impulse armed");
        let source = t.syms[&BTC]
            .pullback_source_day
            .clone()
            .expect("source frozen");
        assert_eq!(original.session, today());
        assert_eq!(
            prod::arm_expiry("BTCUSD", &original.session, original.armed_at),
            at(450)
        );
        t.syms.get_mut(&BTC).unwrap().preopen = Some(current_preparation(&source, current));
        t.prepare(BTC, current, at(250)).await.unwrap();
        let refreshed = t.syms[&BTC].day.as_ref().unwrap().as_ref().unwrap();
        assert_eq!(refreshed.session, current);
        assert_eq!(refreshed.open, Ok(current_open));
        assert_eq!(
            serde_json::to_value(t.syms[&BTC].pullback_source_day.as_ref().unwrap()).unwrap(),
            serde_json::to_value(&source).unwrap(),
            "current peer preparation cannot change original source quote, band or macro"
        );
        assert_eq!(t.syms[&BTC].pullback.unwrap().origin, OPEN);

        // A sibling's status must derive peer movement from the current Day,
        // while the BTC arm still has its original-window opening midpoint.
        let peer = BTC + 1;
        let mut sibling = t.syms[&BTC].clone();
        sibling.id = peer;
        sibling.name = "ETHUSD".into();
        sibling.pullback = Some(mp::ArmedPullback {
            session: current,
            ..original
        });
        t.syms.insert(peer, sibling);
        for (movement, age, has_quote, expect_hold) in [
            (0.003, 0, true, true),
            (0.002, 0, true, false),
            (-0.003, 0, true, false),
            (0.003, 7, true, false),
            (0.003, 0, false, false),
        ] {
            let btc = t.syms.get_mut(&BTC).unwrap();
            let mid = current_open * (1.0 + movement);
            btc.bid = has_quote.then_some(mid - 2.5);
            btc.ask = has_quote.then_some(mid + 2.5);
            btc.bid_at = Some(at(250) - Duration::seconds(age));
            btc.ask_at = btc.bid_at;
            let visible = t.trigger_status(peer, at(250));
            let veto = visible["entry_gates"]
                .as_array()
                .unwrap()
                .iter()
                .any(|g| g["code"] == "peer_hold_below_24bps_or_opposing");
            assert_eq!(!veto, expect_hold, "{visible}");
            if has_quote && age == 0 {
                assert!(
                    (visible["peer_signed_bps"].as_f64().unwrap() - movement * 10000.0).abs()
                        < 1e-8
                );
            }
        }

        // Restart first with the new durable snapshot, then with a pre-upgrade
        // receipt. Both restore source independently of the current Day cache.
        for legacy in [false, true] {
            if legacy {
                // The evidence ledger is append-only, including in fixtures.
                sqlx::query("INSERT INTO trading_decisions (account_id,symbol,kind,reason,detail,settings_version) SELECT account_id,symbol,kind,reason,detail-'source_day',settings_version FROM trading_decisions WHERE account_id=$1 AND symbol='BTCUSD' AND kind='s3_pullback_state' ORDER BY id DESC LIMIT 1")
                    .bind(DEMO).execute(&pool).await.unwrap();
            }
            t.inner = strategy3::Trader::new(&pool, t.allowlist, Environment::Demo, t.session)
                .await
                .unwrap();
            t.reconcile(at(250)).await.unwrap();
            t.syms.get_mut(&BTC).unwrap().preopen = Some(current_preparation(&source, current));
            t.on_quote(BTC, Some(OPEN + 597.5), Some(OPEN + 602.5), at(250))
                .await
                .unwrap();
            let btc = &t.syms[&BTC];
            let arm = btc
                .pullback
                .expect("carried arm restored before original 4h expiry");
            assert_eq!(arm.origin, original.origin);
            assert_eq!(arm.extreme, original.extreme);
            assert_eq!(arm.level, original.level);
            assert_eq!(arm.session, original.session);
            let restored = btc.pullback_source_day.as_ref().unwrap();
            assert_eq!(restored.session, source.session);
            assert_eq!(restored.opening_quote, source.opening_quote);
            assert_eq!(restored.table.sessions_used, source.table.sessions_used);
            assert_eq!(
                restored.table.marks.keys().collect::<Vec<_>>(),
                source.table.marks.keys().collect::<Vec<_>>()
            );
            for (mark, expected) in &source.table.marks {
                match (&restored.table.marks[mark], expected) {
                    // PostgreSQL JSONB decimal encoding can round one ULP;
                    // the existing source validator uses this same bound.
                    (Ok(actual), Ok(expected)) => {
                        assert!((actual - expected).abs() <= 1e-12 * expected.abs().max(1.0))
                    }
                    (actual, expected) => assert_eq!(actual, expected),
                }
            }
            assert_eq!(btc.day.as_ref().unwrap().as_ref().unwrap().session, current);
            assert_eq!(
                btc.day.as_ref().unwrap().as_ref().unwrap().open,
                Ok(current_open)
            );
            assert!(
                t.trigger_status(BTC, at(250))["entry_preparation"]["source_refusal"].is_null()
            );
        }
        let wire = log.lock().unwrap();
        assert!(wire.orders.is_empty() && wire.closes.is_empty() && wire.amends.is_empty());
    });
}
