//! Bounded existing evidence for current-position management. No ownership is
//! inferred here: the application decodes broker receipts and qualifies it.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct Opening {
    pub entry_key: String,
    pub client_msg_id: String,
    pub symbol_id: Option<i64>,
    pub side: Option<String>,
    pub volume: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Receipt {
    pub kind: String,
    pub entry_key: Option<String>,
    pub client_msg_id: Option<String>,
    pub payload_type: i32,
    pub raw: Vec<u8>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Evidence {
    pub openings: Vec<Opening>,
    pub receipts: Vec<Receipt>,
    pub deals: Vec<Vec<u8>>,
    pub truncated: bool,
}

const LIMIT: i64 = 2048;

/// Read only the relevant position, plus opening instructions directly linked
/// by an execution's entry key/message id. History is not required to have
/// caught up with a valid execution receipt.
pub async fn position_evidence(
    pool: &PgPool,
    account: i64,
    position: i64,
) -> Result<Evidence, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    let rows = sqlx::query(
        "SELECT DISTINCT i.entry_key,i.client_msg_id,i.symbol_id,i.side,i.volume,i.created_at
         FROM trading_intents i JOIN trading_events e
           ON (e.entry_key=i.entry_key OR e.client_msg_id=i.client_msg_id)
         WHERE i.account_id=$1 AND e.account_id=$1 AND e.position_id=$2 AND i.kind='open'
         ORDER BY i.created_at LIMIT 33",
    )
    .bind(account)
    .bind(position)
    .fetch_all(&mut *tx)
    .await?;
    let openings = rows.iter().map(opening).collect::<Vec<_>>();
    let receipts = sqlx::query(
        "WITH quantity AS (
            SELECT kind,entry_key,client_msg_id,payload_type,raw,received_at FROM trading_events
            WHERE account_id=$1 AND position_id=$2 AND payload_type<>0
              AND kind IN ('ORDER_FILLED','ORDER_PARTIAL_FILL','RECONCILED_FILL','RECONCILED_PARTIAL_FILL','AUTHORITY_ORDER_OBSERVATION')
              AND (deal_id IS NOT NULL OR kind IN ('RECONCILED_FILL','RECONCILED_PARTIAL_FILL','AUTHORITY_ORDER_OBSERVATION'))
            ORDER BY id LIMIT $3
         ), watermark AS (
            SELECT DISTINCT ON(kind) kind,entry_key,client_msg_id,payload_type,raw,received_at FROM trading_events
            WHERE account_id=$1 AND position_id=$2 AND kind IN ('AUTHORITY_GAP','AUTHORITY_RECOVERY_COMPLETE','AUTHORITY_DISAPPEARED')
            ORDER BY kind,received_at DESC
         ) SELECT * FROM quantity UNION ALL SELECT * FROM watermark",
    )
    .bind(account)
    .bind(position)
    .bind(LIMIT + 1)
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|r| Receipt {
        kind: r.get("kind"),
        entry_key: r.get("entry_key"),
        client_msg_id: r.get("client_msg_id"),
        payload_type: r.get("payload_type"),
        raw: r.get("raw"),
        received_at: r.get("received_at"),
    })
    .collect::<Vec<_>>();
    let deals: Vec<(Vec<u8>,)> = sqlx::query_as(
        "SELECT raw FROM deals WHERE account_id=$1 AND position_id=$2
         ORDER BY executed_at,deal_id LIMIT $3",
    )
    .bind(account)
    .bind(position)
    .bind(LIMIT + 1)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    let truncated = openings.len() > 32
        || receipts.iter().filter(|r| r.payload_type != 0).count() > LIMIT as usize
        || deals.len() > LIMIT as usize;
    Ok(Evidence {
        openings,
        receipts,
        deals: deals.into_iter().map(|r| r.0).collect(),
        truncated,
    })
}

fn opening(r: &sqlx::postgres::PgRow) -> Opening {
    Opening {
        entry_key: r.get("entry_key"),
        client_msg_id: r.get("client_msg_id"),
        symbol_id: r.get("symbol_id"),
        side: r.get("side"),
        volume: r.get("volume"),
        created_at: r.get("created_at"),
    }
}

/// Recovery candidates match the exact wire client-order id, including its
/// protocol length bound. More than one match remains ambiguous, never first.
pub async fn openings_for_wire_key(
    pool: &PgPool,
    account: i64,
    symbol: i64,
    wire_key: &str,
) -> Result<Vec<Opening>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT entry_key,client_msg_id,symbol_id,side,volume,created_at
         FROM trading_intents WHERE account_id=$1 AND symbol_id=$2 AND kind='open'
           AND LEFT(entry_key,50)=$3 ORDER BY id LIMIT 2",
    )
    .bind(account)
    .bind(symbol)
    .bind(wire_key)
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(opening).collect())
}
