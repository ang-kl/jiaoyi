//! One attributable outcome per durably consumed trigger, independent of idle
//! diagnostic deduplication. Absence of this receipt means unresolved, not refused.
use super::*;

#[derive(Debug, Serialize)]
pub(super) struct TriggerOutcome {
    status: &'static str,
    reason: String,
    execution_resolution: &'static str,
    entry_key: Option<String>,
    broker_outcome: Option<Value>,
    error: Option<String>,
}

impl TriggerOutcome {
    pub(super) fn pending() -> Self {
        Self {
            status: "pending",
            reason: "dispatch has not returned".into(),
            execution_resolution: "unresolved",
            entry_key: None,
            broker_outcome: None,
            error: None,
        }
    }

    pub(super) fn refuse(&mut self, reason: impl Into<String>) {
        self.status = "refused";
        self.reason = reason.into();
        self.execution_resolution = "not_sent";
    }

    pub(super) fn sending(&mut self, key: &str) {
        self.entry_key = Some(key.into());
        self.status = "dispatch_attempted";
        self.reason = "dispatch outcome not yet known".into();
        self.execution_resolution = "unresolved";
    }

    pub(super) fn sent(&mut self, sent: &gate::Sent) {
        self.broker_outcome = Some(json!(sent.outcome));
        (self.status, self.execution_resolution) = match &sent.outcome {
            Outcome::Filled { .. } => ("filled", "broker_answered"),
            _ if partly_filled(sent) => ("partly_filled", "partial_fill_requires_reconciliation"),
            Outcome::Rejected { .. } => ("broker_rejected", "broker_answered"),
            Outcome::Unconfirmed => ("unconfirmed", "unresolved"),
            Outcome::Amended { .. } => ("unexpected_broker_answer", "unresolved"),
        };
        self.reason = format!("{:?}", sent.outcome);
    }

    pub(super) fn finish(&mut self, result: &anyhow::Result<()>) {
        if let Err(error) = result {
            self.error = Some(format!("{error:#}"));
            // Preserve a known broker result when later fill persistence or
            // reconciliation fails. An error alone never proves no execution.
            if self.broker_outcome.is_none() {
                self.status = "error";
                self.reason = format!("{error:#}");
                self.execution_resolution = "unresolved";
            }
        } else if matches!(self.status, "pending" | "dispatch_attempted") {
            self.status = "error";
            self.reason = "dispatch returned without an explicit outcome".into();
            self.execution_resolution = "unresolved";
        }
    }
}

impl Decisions<'_> {
    pub(super) async fn trigger_outcome(
        &self,
        symbol: &str,
        consumed_decision_id: i64,
        outcome: &TriggerOutcome,
    ) -> anyhow::Result<()> {
        // Deliberately bypass note(): repeated refusals belong to different
        // consumed triggers. A failed insert propagates; a crash leaves the
        // consumption row unmatched rather than inventing a terminal result.
        ts::record_decision(
            self.pool,
            self.account_id,
            Some(symbol),
            "s3_trigger_outcome",
            &outcome.reason,
            &json!({"policy":"s3_consumed_trigger_outcome_18425_v1",
                "consumed_decision_id":consumed_decision_id,
                "strategy":STRATEGY,"label":LABEL,"prereg_hash":self.hash,
                "outcome":outcome}),
            self.version,
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn repeated_preintent_refusals_each_have_attributable_outcome(pool: PgPool) {
        jiaoyi_store::upsert_account(&pool, 49236051, Environment::Demo)
            .await
            .unwrap();
        let mut decisions = Decisions::new(&pool, 49236051);
        for _ in 0..3 {
            let consumed = ts::record_decision_returning_id(
                &pool,
                49236051,
                Some("ETHUSD"),
                "s3_pullback_state",
                "pullback_trigger_consumed",
                &json!({"label":LABEL}),
                None,
            )
            .await
            .unwrap();
            decisions
                .note(
                    Some("ETHUSD"),
                    "refused",
                    "pair_risk_reservation_unknown",
                    json!({}),
                )
                .await;
            let mut outcome = TriggerOutcome::pending();
            outcome.refuse("pair_risk_reservation_unknown");
            outcome.finish(&Ok(()));
            decisions
                .trigger_outcome("ETHUSD", consumed, &outcome)
                .await
                .unwrap();
        }
        let counts: (i64, i64, i64) = sqlx::query_as(
            "SELECT count(*), count(*) FILTER (WHERE outcomes.n = 0),
                    count(*) FILTER (WHERE outcomes.n <> 1)
             FROM trading_decisions consumed
             CROSS JOIN LATERAL (
                SELECT count(*) AS n FROM trading_decisions receipt
                WHERE receipt.kind='s3_trigger_outcome'
                  AND receipt.account_id=consumed.account_id
                  AND receipt.detail->>'consumed_decision_id'=consumed.id::text
             ) outcomes
             WHERE consumed.kind='s3_pullback_state'
               AND consumed.reason='pullback_trigger_consumed'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(counts, (3, 0, 0));
        let idle_count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM trading_decisions WHERE kind='s3_refused'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            idle_count, 1,
            "idle diagnostics retain their existing deduplication"
        );
    }

    #[test]
    fn errors_and_unknown_answers_never_claim_no_execution() {
        let mut outcome = TriggerOutcome::pending();
        outcome.sending("s3:account:symbol:window:attempt");
        outcome.finish(&Err(anyhow::anyhow!("execution evidence write failed")));
        assert_eq!(outcome.status, "error");
        assert_eq!(outcome.execution_resolution, "unresolved");
        let sent = gate::Sent {
            client_msg_id: "test".into(),
            outcome: Outcome::Filled {
                position_id: Some(42),
                price: Some(100.0),
                volume: Some(1),
            },
            events: vec![],
        };
        outcome.sent(&sent);
        outcome.finish(&Err(anyhow::anyhow!("reconciliation failed")));
        assert_eq!(outcome.status, "filled");
        assert!(outcome.error.is_some());
        outcome.sent(&gate::Sent {
            outcome: Outcome::Unconfirmed,
            ..sent
        });
        outcome.finish(&Ok(()));
        assert_eq!(outcome.status, "unconfirmed");
        assert_eq!(outcome.execution_resolution, "unresolved");
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn pending_consumption_after_crash_has_no_fabricated_outcome(pool: PgPool) {
        jiaoyi_store::upsert_account(&pool, 49236051, Environment::Demo)
            .await
            .unwrap();
        ts::record_decision_returning_id(
            &pool,
            49236051,
            Some("BTCUSD"),
            "s3_pullback_state",
            "pullback_trigger_consumed",
            &json!({"label":LABEL}),
            None,
        )
        .await
        .unwrap();
        let outcomes: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM trading_decisions WHERE kind='s3_trigger_outcome'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(outcomes, 0);
    }
}
