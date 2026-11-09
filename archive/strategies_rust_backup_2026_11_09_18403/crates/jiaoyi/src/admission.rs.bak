//! One admission per account, shared by every strategy loop (Codex P1 on
//! #15). The owner's live rule "one minimum trade at a time" is
//! account-wide. Strategy 1 and Strategy 3 run separate loops on separate
//! connections, so neither loop's own book can enforce it.
//!
//! A live broker-minimum entry takes the account's lock, reads the broker
//! fresh, and goes only while the account holds no position, pending order or
//! unresolved open of this program (any `PROGRAM_LABELS` label, any `s1:` or
//! `s3:` intent). The lock is held through the send, so a second loop sees
//! the first entry as a position or an unresolved intent.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

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
