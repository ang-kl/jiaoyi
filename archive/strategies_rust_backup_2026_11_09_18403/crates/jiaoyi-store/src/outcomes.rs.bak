//! One per-position qualification boundary for outcomes and goal reports.
//! Existing broker/intent facts only: no source rewriting or risk policy.
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::trading::{position_money, ClosedTrade, DealMoney, FILL_KINDS};

#[derive(Debug, Default)]
pub(crate) struct Evidence {
    pub trades: Vec<ClosedTrade>,
    pub unmeasured: usize,
    pub reasons: Vec<String>,
    pub latest_exit: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct Facts {
    position_id: i64,
    entry_key: String,
    requested_volume: Option<i64>,
    risk: Option<f64>,
    receipt_opened: i64,
    opened: i64,
    history_opened: i64,
    closed: i64,
    missing_costs: bool,
    invalid_status: bool,
    unowned_opening: bool,
    latest_history_close: Option<DateTime<Utc>>,
    observed_closed: i64,
    latest_exit: Option<DateTime<Utc>>,
    receipt_deal_missing: bool,
    opening_receipt_deal_missing: bool,
    unmatched_owned_fill: bool,
}

const FACTS_SQL: &str = r#"
WITH owned AS (
 SELECT DISTINCT ON (e.position_id) e.position_id, i.entry_key, i.volume AS requested_volume, i.risk, i.side, i.symbol_id
 FROM trading_intents i JOIN trading_events e
   ON e.entry_key = i.entry_key AND e.account_id = i.account_id
 WHERE i.account_id = $1 AND i.kind = 'open' AND i.entry_key LIKE $2 || ':%'
   AND e.position_id IS NOT NULL AND e.kind = ANY($3)
 ORDER BY e.position_id, (e.volume > 0) DESC NULLS LAST, i.id
), opening_deal_events AS (
 SELECT position_id, SUM(volume)::bigint AS quantity FROM (
   SELECT DISTINCT ON (e.position_id, e.deal_id) e.position_id, e.volume
   FROM trading_events e JOIN owned o ON o.entry_key = e.entry_key AND o.position_id = e.position_id
   WHERE e.account_id = $1 AND e.kind = ANY($3) AND e.volume > 0 AND e.deal_id IS NOT NULL
   ORDER BY e.position_id, e.deal_id, e.id DESC
 ) d GROUP BY position_id
), opening_events AS (
 SELECT o.position_id, GREATEST(COALESCE(d.quantity,0),COALESCE((
  SELECT MAX(e.volume) FROM trading_events e WHERE e.account_id=$1
   AND e.entry_key=o.entry_key AND e.position_id=o.position_id
   AND e.kind=ANY($3) AND e.volume>0
 ),0))::bigint AS quantity
 FROM owned o LEFT JOIN opening_deal_events d ON d.position_id=o.position_id
), owned_orders AS (
 SELECT DISTINCT o.position_id, COALESCE(e.order_id,d.order_id) AS order_id
 FROM owned o JOIN trading_events e ON e.account_id=$1 AND e.entry_key=o.entry_key
  AND e.position_id=o.position_id AND e.kind=ANY($3) AND e.volume>0
 LEFT JOIN deals d ON d.account_id=$1 AND d.position_id=o.position_id
  AND d.deal_id=e.deal_id AND NOT d.is_closing AND d.filled_volume>0
), history AS (
 SELECT o.position_id,
   COALESCE(SUM(d.filled_volume) FILTER (WHERE NOT d.is_closing), 0)::bigint AS opened,
   COALESCE(SUM(d.filled_volume) FILTER (WHERE d.is_closing), 0)::bigint AS closed,
   BOOL_OR(d.is_closing AND (d.close_gross_profit IS NULL OR d.close_swap IS NULL
      OR d.close_commission IS NULL OR d.close_pnl_conversion_fee IS NULL OR COALESCE(d.close_money_digits,d.money_digits) IS NULL
      OR COALESCE(d.close_money_digits,d.money_digits) NOT BETWEEN 0 AND 18)) AS missing_costs,
   BOOL_OR(d.deal_status NOT IN ('FILLED','PARTIALLY_FILLED')) AS invalid_status,
   BOOL_OR(NOT d.is_closing AND NOT COALESCE((
     LOWER(d.trade_side)=o.side AND d.broker_symbol_id=o.symbol_id AND (
       EXISTS (SELECT 1 FROM trading_events e WHERE e.account_id=$1
        AND e.entry_key=o.entry_key AND e.position_id=o.position_id
        AND e.kind=ANY($3) AND e.volume>0 AND e.deal_id=d.deal_id)
       OR EXISTS (SELECT 1 FROM owned_orders x WHERE x.position_id=o.position_id
        AND x.order_id=d.order_id)
     )),false)) AS unowned_opening,
   MAX(d.executed_at) FILTER (WHERE d.is_closing) AS latest_history_close
 FROM owned o LEFT JOIN deals d ON d.account_id = $1 AND d.position_id = o.position_id AND d.filled_volume > 0
 GROUP BY o.position_id
), local_receipts AS (
 SELECT position_id, entry_key, SUM(volume)::bigint AS quantity, MAX(received_at) AS at FROM (
  SELECT DISTINCT ON (e.position_id,e.deal_id) e.position_id, i.entry_key, e.volume,e.received_at
  FROM trading_events e JOIN trading_intents i ON i.entry_key = e.entry_key AND i.account_id=e.account_id
  JOIN owned o ON o.position_id=e.position_id
  WHERE e.account_id=$1 AND i.kind='close' AND e.kind=ANY($3) AND e.volume>0 AND e.deal_id IS NOT NULL
  ORDER BY e.position_id,e.deal_id,e.id DESC
 ) d GROUP BY position_id,entry_key
), fills AS (
 SELECT DISTINCT ON (f.entry_key) f.position_id,f.entry_key,f.volume AS quantity,
   COALESCE(f.answered_at,f.sent_at) AS at
 FROM strategy3_fills f JOIN owned o ON o.position_id=f.position_id
 WHERE $2='s3' AND f.account_id=$1 AND f.kind IN ('broker_exit','tp1','session_end')
   AND f.outcome IN ('filled','partly_filled') AND f.volume>0
 ORDER BY f.entry_key,f.id DESC
), s1_fills AS (
 SELECT DISTINCT ON (f.entry_key) f.position_id,f.entry_key,f.volume AS quantity,
   COALESCE(f.answered_at,f.sent_at) AS at
 FROM strategy1_fills f JOIN owned o ON o.position_id=f.position_id
 WHERE $2='s1' AND f.account_id=$1 AND f.kind='broker_exit'
   AND f.outcome IN ('filled','partly_filled') AND f.volume>0
 ORDER BY f.entry_key,f.id DESC
), grouped_receipts AS (
 SELECT position_id,entry_key,MAX(quantity) AS quantity, MAX(at) AS at
 FROM (SELECT * FROM local_receipts UNION ALL SELECT * FROM fills UNION ALL SELECT * FROM s1_fills) r
 GROUP BY position_id,entry_key
), closing_receipts AS (
 SELECT position_id,SUM(quantity)::bigint AS quantity,MAX(at) AS at FROM grouped_receipts GROUP BY position_id
)
SELECT o.position_id, o.entry_key, o.requested_volume, o.risk,
 COALESCE(e.quantity,0) AS receipt_opened, GREATEST(COALESCE(e.quantity,0),h.opened) AS opened,
 h.opened AS history_opened,h.closed,COALESCE(h.missing_costs,false) AS missing_costs,
 COALESCE(h.invalid_status,false) AS invalid_status,
 COALESCE(h.unowned_opening,false) AS unowned_opening, h.latest_history_close,
 COALESCE(c.quantity,0) AS observed_closed, c.at AS latest_exit,
 (EXISTS (
  SELECT 1 FROM trading_events x JOIN trading_intents i ON i.account_id=x.account_id AND i.entry_key=x.entry_key
  WHERE x.account_id=$1 AND x.position_id=o.position_id AND i.kind='close'
   AND x.kind=ANY($3) AND x.volume>0 AND x.deal_id IS NOT NULL
   AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=o.position_id AND d.deal_id=x.deal_id AND d.is_closing AND d.filled_volume>=x.volume)
 ) OR EXISTS (
  SELECT 1 FROM grouped_receipts g WHERE g.position_id=o.position_id AND g.entry_key LIKE $2||'-broker:%'
   AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=o.position_id AND d.is_closing
     AND d.filled_volume>=g.quantity AND g.entry_key=$2||'-broker:'||$1::text||':'||o.position_id::text||':'||d.deal_id::text)
 )) AS receipt_deal_missing,
 EXISTS (
  SELECT 1 FROM trading_events x WHERE x.account_id=$1 AND x.position_id=o.position_id
   AND x.entry_key=o.entry_key AND x.kind=ANY($3) AND x.volume>0 AND x.deal_id IS NOT NULL
   AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=o.position_id
    AND d.deal_id=x.deal_id AND NOT d.is_closing AND d.filled_volume>=x.volume)
 ) AS opening_receipt_deal_missing,
 EXISTS (
  SELECT 1 FROM trading_events x WHERE x.account_id=$1 AND x.position_id=o.position_id
   AND x.kind=ANY($3) AND x.volume>0 AND x.entry_key IS NULL
   AND NOT EXISTS (SELECT 1 FROM deals d WHERE d.account_id=$1 AND d.position_id=x.position_id
      AND d.deal_id=x.deal_id AND d.filled_volume>=x.volume)
   AND NOT EXISTS (SELECT 1 FROM grouped_receipts g WHERE g.position_id=x.position_id
      AND g.quantity>=x.volume AND g.entry_key=$2||'-broker:'||$1::text||':'||x.position_id::text||':'||x.deal_id::text)
 ) AS unmatched_owned_fill
FROM owned o JOIN history h ON h.position_id=o.position_id
LEFT JOIN opening_events e ON e.position_id=o.position_id
LEFT JOIN closing_receipts c ON c.position_id=o.position_id
"#;

async fn snapshot(pool: &PgPool) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

async fn facts(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
    strategy: &str,
) -> Result<Vec<Facts>, sqlx::Error> {
    sqlx::query_as(FACTS_SQL)
        .bind(account_id)
        .bind(strategy)
        .bind(FILL_KINDS)
        .fetch_all(&mut **tx)
        .await
}

async fn money(
    tx: &mut Transaction<'_, Postgres>,
    account_id: i64,
    positions: &[i64],
) -> Result<HashMap<i64, Vec<DealMoney>>, sqlx::Error> {
    if positions.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        "SELECT position_id, is_closing, filled_volume, executed_at, money_digits,
                close_gross_profit, close_swap, close_commission, close_pnl_conversion_fee,
                close_money_digits
         FROM deals WHERE account_id = $1 AND position_id = ANY($2) AND filled_volume > 0
         ORDER BY position_id, executed_at, deal_id",
    )
    .bind(account_id)
    .bind(positions)
    .fetch_all(&mut **tx)
    .await?;
    let mut out: HashMap<i64, Vec<DealMoney>> = HashMap::new();
    for row in rows {
        out.entry(row.get("position_id"))
            .or_default()
            .push(DealMoney {
                is_closing: row.get("is_closing"),
                filled_volume: row.get("filled_volume"),
                executed_at: row.get("executed_at"),
                money_digits: row.get("money_digits"),
                close_gross_profit: row.get("close_gross_profit"),
                close_swap: row.get("close_swap"),
                close_commission: row.get("close_commission"),
                close_pnl_conversion_fee: row.get("close_pnl_conversion_fee"),
                close_money_digits: row.get("close_money_digits"),
            });
    }
    Ok(out)
}

enum PositionState {
    Open,
    Incomplete(Vec<String>),
    Complete(ClosedTrade),
}

fn classify(f: &Facts, deals: &[DealMoney], cutoff: DateTime<Utc>) -> PositionState {
    let mut reasons = Vec::new();
    let terminal = f.observed_closed > 0 && f.observed_closed >= f.opened;
    let history_complete = f.history_opened > 0 && f.closed >= f.history_opened;
    if f.receipt_opened <= 0 && (f.closed > 0 || terminal) {
        reasons.push("no positive owned opening receipt".into());
    }
    if f.closed > f.history_opened {
        reasons.push("closing history exceeds proven opening quantity".into());
    }
    if (f.history_opened < f.opened || f.opening_receipt_deal_missing) && (f.closed > 0 || terminal)
    {
        reasons.push(
            "opening history trails confirmed opening receipt quantity or deal identity".into(),
        );
    }
    if terminal && (f.opened == 0 || f.closed < f.observed_closed || f.receipt_deal_missing) {
        reasons.push("terminal exit receipts await complete broker history".into());
    } else if history_complete && f.receipt_deal_missing {
        reasons.push("closing history does not match an observed exit receipt".into());
    }
    if f.unmatched_owned_fill {
        reasons.push("unclassified owned fill awaits authoritative history".into());
    }
    if f.unowned_opening {
        reasons.push("an opening increment lacks original owned-order provenance".into());
    }
    if f.invalid_status {
        reasons.push("positive-volume history has an unsupported deal status".into());
    }
    if f.latest_history_close.is_some_and(|at| at > cutoff) {
        reasons.push("a broker close timestamp is after the assessment cutoff".into());
    }
    if f.latest_exit.is_some_and(|at| at > cutoff) {
        reasons.push("an exit receipt timestamp is after the assessment cutoff".into());
    }
    if history_complete || terminal {
        if f.requested_volume.is_none_or(|v| v <= 0 || v < f.opened) {
            reasons.push(
                "original requested volume missing or inconsistent with proven opening quantity"
                    .into(),
            );
        }
        if f.missing_costs {
            reasons.push("closing costs or monetary precision missing".into());
        }
    }
    if !reasons.is_empty() {
        return PositionState::Incomplete(reasons);
    }
    if !history_complete {
        return PositionState::Open;
    }
    let (opened, closed_at, net) = match position_money(deals) {
        Ok(value) => value,
        Err(reason) => return PositionState::Incomplete(vec![reason]),
    };
    let share = opened as f64 / f.requested_volume.expect("qualified request") as f64;
    let Some(risk) = f
        .risk
        .map(|v| v * share)
        .filter(|v| v.is_finite() && *v > 0.0)
    else {
        return PositionState::Incomplete(vec![
            "original filled-share risk is missing or invalid".into()
        ]);
    };
    let r = net / risk;
    if !r.is_finite() {
        return PositionState::Incomplete(vec!["net-R is not finite".into()]);
    }
    PositionState::Complete(ClosedTrade {
        entry_key: f.entry_key.clone(),
        position_id: f.position_id,
        closed_at,
        r,
    })
}

fn qualify(
    rows: Vec<Facts>,
    mut deals: HashMap<i64, Vec<DealMoney>>,
    cutoff: DateTime<Utc>,
) -> Evidence {
    let mut out = Evidence::default();
    for f in rows {
        out.latest_exit = out.latest_exit.max(f.latest_exit);
        match classify(
            &f,
            &deals.remove(&f.position_id).unwrap_or_default(),
            cutoff,
        ) {
            PositionState::Open => {}
            PositionState::Incomplete(reasons) => {
                out.unmeasured += 1;
                out.reasons.extend(
                    reasons
                        .into_iter()
                        .map(|reason| format!("position {}: {reason}", f.position_id)),
                );
            }
            PositionState::Complete(trade) => out.trades.push(trade),
        }
    }
    out.trades.sort_by(|a, b| {
        a.closed_at
            .cmp(&b.closed_at)
            .then(a.position_id.cmp(&b.position_id))
    });
    out
}

/// Both reads see one committed database snapshot. A receipt that already
/// precedes its history remains incomplete; a later poll can qualify it.
/// This certifies no broker-history freshness and changes no source record.
pub(crate) async fn read(
    pool: &PgPool,
    account_id: i64,
    strategy: &str,
    cutoff: DateTime<Utc>,
) -> Result<Evidence, sqlx::Error> {
    let mut tx = snapshot(pool).await?;
    let rows = facts(&mut tx, account_id, strategy).await?;
    let positions: Vec<i64> = rows.iter().map(|f| f.position_id).collect();
    let deals = money(&mut tx, account_id, &positions).await?;
    tx.commit().await?;
    Ok(qualify(rows, deals, cutoff))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn receipt_and_money_reads_use_one_snapshot_then_observe_the_revision(pool: PgPool) {
        sqlx::query("INSERT INTO trading_intents(account_id,entry_key,client_msg_id,kind,symbol_id,position_id,side,volume,relative_stop,risk,max_risk) VALUES(77,'s3:77:1','open','open',1,1,'buy',100,100000,1,1),(77,'s3-end:77:1','close','close',1,1,'sell',100,NULL,NULL,1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO trading_events(account_id,entry_key,client_msg_id,payload_type,kind,position_id,order_id,deal_id,symbol_id,volume,raw,received_at) VALUES(77,'s3:77:1','open',2126,'ORDER_FILLED',1,10,10,1,100,''::bytea,now()-interval '1 hour'),(77,'s3-end:77:1','close',2126,'ORDER_FILLED',1,11,11,1,100,''::bytea,now()-interval '1 minute')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO deals(account_id,deal_id,order_id,position_id,broker_symbol_id,volume,filled_volume,trade_side,deal_status,created_at,executed_at,money_digits,is_closing,close_gross_profit,close_swap,close_commission,close_pnl_conversion_fee,close_money_digits,close_closed_volume,raw,fetched_at) VALUES(77,10,10,1,1,100,100,'BUY','FILLED',now()-interval '1 hour',now()-interval '1 hour',2,false,NULL,NULL,NULL,NULL,NULL,NULL,''::bytea,now()),(77,11,11,1,1,100,100,'SELL','FILLED',now()-interval '1 minute',now()-interval '1 minute',2,true,100,0,0,0,2,100,''::bytea,now())")
            .execute(&pool).await.unwrap();
        let cutoff = Utc::now();
        let mut tx = snapshot(&pool).await.unwrap();
        let before = facts(&mut tx, 77, "s3").await.unwrap();
        assert_eq!(before[0].closed, 100);
        // Another committed connection revises history between the two reads.
        sqlx::query(
            "UPDATE deals SET filled_volume=0, fetched_at=now() WHERE account_id=77 AND deal_id=11",
        )
        .execute(&pool)
        .await
        .unwrap();
        let same_snapshot = money(&mut tx, 77, &[1]).await.unwrap();
        tx.commit().await.unwrap();
        let first = qualify(before, same_snapshot, cutoff);
        assert_eq!((first.trades.len(), first.unmeasured), (1, 0));
        assert_eq!(first.trades[0].r, 1.0);
        let next = read(&pool, 77, "s3", Utc::now()).await.unwrap();
        assert_eq!((next.trades.len(), next.unmeasured), (0, 1));
        assert!(next
            .reasons
            .iter()
            .any(|r| r.contains("terminal exit receipts")));
    }
}
