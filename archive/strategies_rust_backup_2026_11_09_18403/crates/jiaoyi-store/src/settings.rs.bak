//! Settings storage: values, per-account overrides, the version, and the
//! append-only change log. A change is validated against its definition and
//! refused with the reason when out of bounds; nothing is clamped.

use jiaoyi_core::settings::{validate, Definition, Kind, Snapshot};
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgPool, Row};

#[derive(Debug, thiserror::Error)]
pub enum ChangeError {
    #[error("unknown setting {0}")]
    UnknownKey(String),
    #[error("refused: {0}")]
    Invalid(String),
    #[error("refused: a reason is required")]
    NoReason,
    #[error("refused: who made the change is required")]
    NoActor,
    #[error("refused: the global value of {0} cannot be removed")]
    GlobalRemoval(String),
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
}

pub async fn definitions(pool: &PgPool) -> Result<Vec<Definition>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT key, kind, min_value, max_value, description FROM setting_definitions ORDER BY key",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            Some(Definition {
                key: r.get("key"),
                kind: Kind::parse(r.get::<&str, _>("kind"))?,
                min: r.get("min_value"),
                max: r.get("max_value"),
                description: r.get("description"),
            })
        })
        .collect())
}

/// The settings in force for `account_id` (global values plus its overrides),
/// or the global values alone for `None`, with the version, read in one snapshot.
pub async fn snapshot(pool: &PgPool, account_id: Option<i64>) -> Result<Snapshot, sqlx::Error> {
    // A single PostgreSQL statement sees one MVCC snapshot, including the
    // version. This retains the old repeatable-read view without six serial
    // commands for every account snapshot. The left join retains an empty
    // settings map; an absent singleton still returns RowNotFound.
    let rows = sqlx::query(
        "SELECT v.version, s.key, s.value, s.account_id
         FROM settings_version v LEFT JOIN settings s
           ON s.account_id IS NULL OR s.account_id = $1",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;
    let version = rows.first().ok_or(sqlx::Error::RowNotFound)?.get("version");
    let mut global = Vec::new();
    let mut account = Vec::new();
    for row in rows {
        let Some(key) = row.get::<Option<String>, _>("key") else {
            continue;
        };
        let value = row.get("value");
        match row.get::<Option<i64>, _>("account_id") {
            Some(id) => account.push((key, value, id)),
            None => global.push((key, value)),
        }
    }
    Ok(Snapshot::merge(version, global, account))
}

/// Set a value (global for `account_id = None`), or remove an account
/// override (`new_value = None`). Returns the new settings version.
pub async fn change(
    pool: &PgPool,
    key: &str,
    account_id: Option<i64>,
    new_value: Option<Value>,
    changed_by: &str,
    reason: &str,
) -> Result<i64, ChangeError> {
    if changed_by.trim().is_empty() {
        return Err(ChangeError::NoActor);
    }
    if reason.trim().is_empty() {
        return Err(ChangeError::NoReason);
    }
    let def = definitions(pool)
        .await?
        .into_iter()
        .find(|d| d.key == key)
        .ok_or_else(|| ChangeError::UnknownKey(key.to_string()))?;
    match &new_value {
        Some(v) => validate(&def, v).map_err(ChangeError::Invalid)?,
        None if account_id.is_none() => return Err(ChangeError::GlobalRemoval(key.into())),
        None => {}
    }

    let mut tx = pool.begin().await?;
    let version: i64 =
        sqlx::query_scalar("UPDATE settings_version SET version = version + 1 RETURNING version")
            .fetch_one(&mut *tx)
            .await?;
    let old: Option<Value> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = $1 AND COALESCE(account_id, 0) = COALESCE($2, 0) FOR UPDATE",
    )
    .bind(key)
    .bind(account_id)
    .fetch_optional(&mut *tx)
    .await?;
    match &new_value {
        Some(v) => {
            sqlx::query(
                "INSERT INTO settings (key, account_id, value, version) VALUES ($1, $2, $3, $4)
                 ON CONFLICT (key, COALESCE(account_id, 0))
                 DO UPDATE SET value = EXCLUDED.value, version = EXCLUDED.version, updated_at = now()",
            )
            .bind(key)
            .bind(account_id)
            .bind(v)
            .bind(version)
            .execute(&mut *tx)
            .await?;
        }
        None => {
            sqlx::query("DELETE FROM settings WHERE key = $1 AND account_id = $2")
                .bind(key)
                .bind(account_id)
                .execute(&mut *tx)
                .await?;
        }
    }
    sqlx::query(
        "INSERT INTO settings_changes (version, key, account_id, old_value, new_value, changed_by, reason)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(version)
    .bind(key)
    .bind(account_id)
    .bind(old)
    .bind(new_value)
    .bind(changed_by.trim())
    .bind(reason.trim())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(version)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ChangeRow {
    pub version: i64,
    pub key: String,
    pub account_id: Option<i64>,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub changed_by: String,
    pub reason: String,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

pub async fn history(pool: &PgPool, key: Option<&str>) -> Result<Vec<ChangeRow>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT version, key, account_id, old_value, new_value, changed_by, reason, changed_at
         FROM settings_changes WHERE $1::text IS NULL OR key = $1 ORDER BY version DESC",
    )
    .bind(key)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ChangeRow {
            version: r.get("version"),
            key: r.get("key"),
            account_id: r.get("account_id"),
            old_value: r.get("old_value"),
            new_value: r.get("new_value"),
            changed_by: r.get("changed_by"),
            reason: r.get("reason"),
            changed_at: r.get("changed_at"),
        })
        .collect())
}
