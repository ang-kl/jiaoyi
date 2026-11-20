//! One admission per account, shared by every strategy loop (Codex P1 on
//! #15). Strategy 1's live minimum rule is account-wide. Strategy 3 has its
//! own two-slot limit (No. 18541).
//!
//! Live broker-minimum entries read the broker and ledger under the account's
//! lock. The lock is held through the send so another loop sees the entry as
//! a position or an unresolved intent.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use jiaoyi_ctrader::messages::ProtoOaReconcileRes;
use jiaoyi_ctrader::trading::PROGRAM_LABELS;
use jiaoyi_ctrader::AccountSession;
use jiaoyi_store::sqlx::PgPool;
use jiaoyi_store::trading as ts;
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

type Locks = Mutex<HashMap<i64, Arc<AsyncMutex<()>>>>;

static LOCKS: OnceLock<Locks> = OnceLock::new();

/// The account's admission lock, held from the admission check through the
/// send.
pub async fn lock(account_id: i64) -> OwnedMutexGuard<()> {
    let m = {
        let mut g = LOCKS.get_or_init(Default::default).lock().unwrap();
        g.entry(account_id).or_default().clone()
    };
    m.lock_owned().await
}

/// The entry keys of this program's strategies' opens.
pub fn program_intent(entry_key: &str) -> bool {
    entry_key.starts_with("s1:") || entry_key.starts_with("s3:")
}

/// Whether a live broker-minimum entry may go now. The positions and
/// pending orders are read fresh from the broker; the unresolved opens come
/// from the ledger. Call with the account's lock held.
pub async fn live_minimum(pool: &PgPool, session: &AccountSession<'_>) -> Result<(), String> {
    let account_id = session.account().id();
    let rec = session
        .reconcile()
        .await
        .map_err(|e| format!("the open positions are unreadable ({e}): no entry"))?;
    let ours = |l: &Option<String>| l.as_deref().is_some_and(|l| PROGRAM_LABELS.contains(&l));
    let positions = rec
        .position
        .iter()
        .filter(|p| ours(&p.trade_data.label))
        .count();
    let orders = rec
        .order
        .iter()
        .filter(|o| ours(&o.trade_data.label))
        .count();
    let unresolved = ts::open_intents_unresolved(pool, account_id)
        .await
        .map_err(|e| format!("the unresolved opens are unreadable ({e}): no entry"))?
        .into_iter()
        .filter(|u| program_intent(&u.entry_key))
        .count();
    if positions + orders + unresolved > 0 {
        return Err(format!(
            "one minimum trade at a time, account-wide: {positions} position(s), {orders} pending order(s) and {unresolved} unresolved open(s) of this program on the account"
        ));
    }
    Ok(())
}

/// Check the live Strategy 3 slots from fresh broker positions and durable open
/// intents. Call with the account's lock held through dispatch.
pub async fn live_s3_concurrency(
    pool: &PgPool,
    session: &AccountSession<'_>,
    max_s3_positions: usize,
) -> Result<usize, String> {
    let book = session
        .reconcile()
        .await
        .map_err(|e| format!("the open positions are unreadable ({e}): no entry"))?;
    let unresolved = ts::open_intents_unresolved(pool, session.account().id())
        .await
        .map_err(|e| format!("the unresolved opens are unreadable ({e}): no entry"))?;
    check_s3_concurrency(&book, &unresolved, max_s3_positions)
}

fn check_s3_concurrency(
    book: &ProtoOaReconcileRes,
    unresolved: &[ts::UnresolvedIntent],
    max_s3_positions: usize,
) -> Result<usize, String> {
    let open_positions: HashSet<_> = book
        .position
        .iter()
        .filter(|p| p.trade_data.label.as_deref() == Some(crate::strategy3::LABEL))
        .map(|p| p.position_id)
        .collect();
    // The query excludes terminal intents. A partial fill and its unresolved
    // remainder occupy the same slot when the position is already open.
    let unresolved_opens = unresolved
        .iter()
        .filter(|u| {
            u.entry_key.starts_with("s3:")
                && u.position_id.is_none_or(|id| !open_positions.contains(&id))
        })
        .count();
    let active_s3 = open_positions.len() + unresolved_opens;
    if active_s3 >= max_s3_positions {
        return Err(format!(
            "live: at most {max_s3_positions} Strategy 3 minimum trades at a time, and {active_s3} Strategy 3 position(s)/unresolved open(s) are active on the account"
        ));
    }
    Ok(active_s3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiaoyi_ctrader::messages::{ProtoOaPosition, ProtoOaTradeData};

    #[test]
    fn live_s3_counts_only_exact_labels_and_unmatched_s3_intents() {
        let position = |id, label: Option<&str>| ProtoOaPosition {
            position_id: id,
            trade_data: ProtoOaTradeData {
                label: label.map(str::to_owned),
                ..Default::default()
            },
            ..Default::default()
        };
        let intent = |key: &str, position_id: Option<i64>| ts::UnresolvedIntent {
            entry_key: key.into(),
            client_msg_id: key.into(),
            symbol_id: None,
            volume: Some(100),
            filled: if position_id.is_some() { 50 } else { 0 },
            position_id,
            risk: Some(10.0),
            created_at: chrono::DateTime::UNIX_EPOCH,
        };
        let mut book = ProtoOaReconcileRes {
            position: vec![
                position(1, Some("jiaoyi-s1")),
                position(2, Some("")),
                position(3, None),
                position(4, Some("jiaoyi-s3-other")),
            ],
            ..Default::default()
        };
        let mut unresolved = vec![intent("s1:pending", None)];
        let max = crate::strategy3::LIVE_MAX_S3_POSITIONS;
        assert_eq!(check_s3_concurrency(&book, &unresolved, max), Ok(0));

        book.position
            .push(position(5, Some(crate::strategy3::LABEL)));
        unresolved.push(intent("s3:partial", Some(5)));
        assert_eq!(check_s3_concurrency(&book, &unresolved, max), Ok(1));

        unresolved.push(intent("s3:pending", None));
        assert!(check_s3_concurrency(&book, &unresolved, max)
            .unwrap_err()
            .contains("2 Strategy 3 position(s)/unresolved open(s) are active"));

        unresolved.pop();
        book.position
            .push(position(6, Some(crate::strategy3::LABEL)));
        assert!(check_s3_concurrency(&book, &unresolved, max).is_err());

        book.position.truncate(4);
        assert_eq!(check_s3_concurrency(&book, &unresolved, max), Ok(1));
    }
}
