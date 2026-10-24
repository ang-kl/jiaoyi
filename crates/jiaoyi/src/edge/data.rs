//! One data-access layer per job stage (the held-out seal).
//!
//! A development job reads only data that ENDS before 2025-10-24 00:00 UTC:
//! daily bars whose 24 hours end by then, and ticks before then. A request
//! reaching past that is refused with [`DataError::Sealed`]; it is never
//! clamped or filtered silently. A held-out job reads up to the end of the
//! held-out period (2026-10-23, so until 2026-10-24 00:00 UTC), warm-up
//! history included.

use chrono::{DateTime, Duration, Utc};
use jiaoyi_core::seal;
use jiaoyi_ctrader::tickdata::{QuoteSide, TickFetch};
use jiaoyi_ctrader::{AccountSession, ClientError};
use jiaoyi_store::sqlx::PgPool;
use jiaoyi_store::{self as store, BarRow};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Development,
    HeldOut,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Development => "development",
            Stage::HeldOut => "held_out",
        }
    }

    pub fn parse(s: &str) -> Option<Stage> {
        match s {
            "development" => Some(Stage::Development),
            "held_out" => Some(Stage::HeldOut),
            _ => None,
        }
    }

    /// The instant this stage's data must end by (exclusive).
    pub fn end(self) -> DateTime<Utc> {
        match self {
            Stage::Development => seal::seal_instant(),
            Stage::HeldOut => seal::seal_instant() + Duration::days(365),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("sealed: {0}")]
    Sealed(String),
    #[error(transparent)]
    Store(#[from] store::StoreError),
    #[error(transparent)]
    Client(#[from] ClientError),
}

/// The data a job may read.
#[derive(Clone, Copy)]
pub struct Data<'a> {
    pool: &'a PgPool,
    stage: Stage,
}

impl<'a> Data<'a> {
    pub fn new(pool: &'a PgPool, stage: Stage) -> Self {
        Data { pool, stage }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn pool(&self) -> &'a PgPool {
        self.pool
    }

    /// A symbol's daily bars ending by `end`. Refused when `end` is past
    /// this stage's limit.
    pub async fn daily_bars(
        &self,
        account_id: i64,
        symbol_id: i64,
        end: DateTime<Utc>,
    ) -> Result<Vec<BarRow>, DataError> {
        if end > self.stage.end() {
            return Err(DataError::Sealed(format!(
                "a {} job reads bars ending by {}, not {} (held-out period {} → {}); refused, not clamped",
                self.stage.as_str(),
                self.stage.end().to_rfc3339(),
                end.to_rfc3339(),
                seal::held_out_from(),
                seal::held_out_to()
            )));
        }
        let rows =
            store::research::daily_bars_ending_by(self.pool, account_id, symbol_id, end).await?;
        // Second line: no row may run past the stage's limit.
        if let Some(b) = rows
            .iter()
            .find(|b| seal::daily_bar_end(b.bar_open_utc) > self.stage.end())
        {
            return Err(DataError::Sealed(format!(
                "the bar dated {} runs past {}",
                b.bar_date,
                self.stage.end().to_rfc3339()
            )));
        }
        Ok(rows)
    }

    /// Broker ticks of one side over `[from_ms, to_ms]` (at most a week),
    /// streamed from the broker and never stored. Refused when the window
    /// reaches this stage's limit.
    pub async fn ticks(
        &self,
        session: &AccountSession<'_>,
        symbol_id: i64,
        side: QuoteSide,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<TickFetch, DataError> {
        let end_ms = self.stage.end().timestamp_millis();
        if to_ms >= end_ms || from_ms >= end_ms {
            return Err(DataError::Sealed(format!(
                "a {} job reads ticks before {}, not up to {}; refused, not clamped",
                self.stage.as_str(),
                self.stage.end().to_rfc3339(),
                DateTime::<Utc>::from_timestamp_millis(to_ms)
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_default()
            )));
        }
        Ok(session
            .tick_history(symbol_id, side, from_ms, to_ms)
            .await?)
    }
}
