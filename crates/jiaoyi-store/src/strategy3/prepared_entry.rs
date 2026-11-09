//! One statement linearises prepared admission against owner settings changes.
use super::OrderRow;
use crate::trading::IntentRow;
use sqlx::PgPool;

/// Persist a prepared opening intent and its S3 evidence atomically. The shared
/// settings row lock conflicts with `settings::change`'s version UPDATE. Thus a
/// committed owner change cannot slip between version verification and intent
/// persistence. `false` means the exactly-once intent already exists; never send
/// again. An order write error rolls back the intent in the same statement.
pub async fn prepared_intent_and_order(
    pool: &PgPool,
    i: &IntentRow,
    o: &OrderRow,
) -> Result<bool, sqlx::Error> {
    let expected = i.settings_version.ok_or_else(|| {
        sqlx::Error::Protocol("prepared admission has no settings version".into())
    })?;
    if i.kind != "open"
        || o.kind != "open"
        || i.account_id != o.account_id
        || i.entry_key != o.entry_key
        || i.symbol_id != Some(o.symbol_id)
        || i.position_id != o.position_id
        || i.side != o.side
        || i.volume != o.volume
    {
        return Err(sqlx::Error::Protocol(
            "prepared intent/order identity differs".into(),
        ));
    }
    let (version, inserted, recorded): (Option<i64>, bool, bool) = sqlx::query_as(
        "WITH locked_settings AS MATERIALIZED (
             SELECT version FROM settings_version FOR SHARE
         ), inserted_intent AS (
             INSERT INTO trading_intents (account_id,entry_key,client_msg_id,kind,symbol_id,
                 position_id,side,volume,relative_stop,stop_price,take_profit,risk,max_risk,settings_version)
             SELECT $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14
             FROM locked_settings WHERE version=$14
             ON CONFLICT (entry_key) DO NOTHING RETURNING entry_key
         ), inserted_order AS (
             INSERT INTO strategy3_orders (account_id,entry_key,label,prereg_hash,kind,symbol,
                 symbol_id,position_id,side,session_date,decided_at,bid,ask,volume,detail)
             SELECT $1,$2,$15,$16,$4,$17,$5,$6,$7,$18,$19,$20,$21,$8,$22
             FROM inserted_intent RETURNING entry_key
         )
         SELECT (SELECT version FROM locked_settings),
                EXISTS(SELECT 1 FROM inserted_intent), EXISTS(SELECT 1 FROM inserted_order)",
    )
    .bind(i.account_id)
    .bind(&i.entry_key)
    .bind(&i.client_msg_id)
    .bind(&i.kind)
    .bind(i.symbol_id)
    .bind(i.position_id)
    .bind(&i.side)
    .bind(i.volume)
    .bind(i.relative_stop)
    .bind(i.stop_price)
    .bind(i.take_profit)
    .bind(i.risk)
    .bind(i.max_risk)
    .bind(expected)
    .bind(&o.label)
    .bind(&o.prereg_hash)
    .bind(&o.symbol)
    .bind(o.session_date)
    .bind(o.decided_at)
    .bind(o.bid)
    .bind(o.ask)
    .bind(&o.detail)
    .fetch_one(pool)
    .await?;
    if version != Some(expected) {
        return Err(sqlx::Error::Protocol(
            "prepared admission settings changed or missing; refresh before entry".into(),
        ));
    }
    if inserted != recorded {
        return Err(sqlx::Error::Protocol(
            "prepared intent/order atomicity failed".into(),
        ));
    }
    Ok(inserted)
}
