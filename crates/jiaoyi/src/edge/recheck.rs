//! Held-out trades re-checked tick by tick on their entry and exit days
//! (pre-registration: "the tick result is the one counted").
//!
//! The days a trade can enter on are the daily signals. On such a day the
//! broker's ticks are streamed (never stored):
//! - the fill is the first ask at or above the level for a long (the first
//!   bid at or below it for a short), made worse by the slippage; a day whose
//!   ticks never reach the level makes no trade;
//! - from the fill on, the stop trails tick by tick, stop_atr × the last
//!   completed day's ATR behind the best price since the fill, and a tick
//!   beyond it exits (at that tick's bid for a long, ask for a short, minus or
//!   plus the slippage). Prices are compared in the bars' measured basis.
//!
//! Between those days the daily-bar rules run unchanged. A day on which the
//! daily rules would exit is streamed the same way; if its ticks do not reach
//! the stop, the trade goes on. Both results are kept, with the difference.

use chrono::{DateTime, Duration, Utc};
use jiaoyi_core::research::h1::{
    entry_signal, run_trade, start_index, ExitReason, H1Params, Side, Trade,
};
use jiaoyi_ctrader::tickdata::{HistTick, QuoteSide};
use jiaoyi_ctrader::AccountSession;
use serde::Serialize;

use super::data::{Data, DataError};
use super::h1job::Prepared;
use super::sampler::merge;

/// One trade's daily-bar and tick results.
#[derive(Debug, Clone, Serialize)]
pub struct Recheck {
    pub symbol: String,
    pub entry_date: chrono::NaiveDate,
    pub daily: Option<Trade>,
    pub tick: Option<Trade>,
    /// tick net R − daily net R, when both exist.
    pub diff_r: Option<f64>,
    pub note: Option<String>,
}

/// `(ts_ms, bid, ask)` quote updates of the bar opening at `open`.
async fn day_quotes(
    data: &Data<'_>,
    session: &AccountSession<'_>,
    symbol_id: i64,
    open: DateTime<Utc>,
) -> Result<Vec<(i64, Option<i64>, Option<i64>)>, DataError> {
    let from = open.timestamp_millis();
    let to = (open + Duration::days(1)).timestamp_millis() - 1;
    let b = data
        .ticks(session, symbol_id, QuoteSide::Bid, from, to)
        .await?;
    let a = data
        .ticks(session, symbol_id, QuoteSide::Ask, from, to)
        .await?;
    let bids: Vec<HistTick> = b.ticks.iter().map(|(t, _)| *t).collect();
    let asks: Vec<HistTick> = a.ticks.iter().map(|(t, _)| *t).collect();
    Ok(merge(&bids, &asks, None, None))
}

fn px(e5: i64) -> f64 {
    e5 as f64 / 100_000.0
}

/// The held-out trades of one symbol, re-checked on ticks.
pub async fn simulate_with_ticks(
    data: &Data<'_>,
    session: &AccountSession<'_>,
    x: &Prepared,
    p: &H1Params,
) -> Result<(Vec<Trade>, Vec<Recheck>), DataError> {
    let s = &x.series;
    let ind = &x.indicators;
    let mut trades = Vec::new();
    let mut notes = Vec::new();
    if s.dates.is_empty() {
        return Ok((trades, notes));
    }
    let mut t = start_index(s, p);
    while t <= s.last {
        let Some((side, level)) = entry_signal(s, ind, p, t) else {
            t += 1;
            continue;
        };
        let entry_ref = match side {
            Side::Long => level.max(s.open[t]),
            Side::Short => level.min(s.open[t]),
        };
        let daily = run_trade(s, ind, p, side, t, entry_ref, Some(level));
        let (tick, note) = tick_trade(data, session, x, p, side, t, level).await?;
        notes.push(Recheck {
            symbol: s.symbol.clone(),
            entry_date: s.dates[t],
            diff_r: match (&daily, &tick) {
                (Some(d), Some(k)) => Some(k.net_r - d.net_r),
                _ => None,
            },
            daily,
            tick: tick.clone(),
            note,
        });
        match tick {
            Some(tr) => {
                t = tr.exit_index + 1;
                trades.push(tr);
            }
            None => t += 1,
        }
    }
    Ok((trades, notes))
}

#[allow(clippy::too_many_arguments)]
async fn tick_trade(
    data: &Data<'_>,
    session: &AccountSession<'_>,
    x: &Prepared,
    p: &H1Params,
    side: Side,
    t_e: usize,
    level: f64,
) -> Result<(Option<Trade>, Option<String>), DataError> {
    let s = &x.series;
    let ind = &x.indicators;
    let id = x.costs.symbol_id;
    let long = side == Side::Long;
    let Some(atr_e) = ind.atr[t_e - 1] else {
        return Ok((None, Some("no ATR before the entry day".into())));
    };
    let d = p.stop_atr * atr_e;
    let slip = |k: usize| p.slip_mult * s.spread[k];
    // The side compared with the stop, in the bars' basis: a long's bid
    // plus the bar→bid step; a short's ask less the bar→ask step.
    let reference = |k: usize, bid: i64, ask: i64| {
        if long {
            px(bid) + s.to_bid[k]
        } else {
            px(ask) - s.to_ask[k]
        }
    };
    let q = day_quotes(data, session, id, x.opens[t_e]).await?;
    let fill_at = q.iter().position(|(_, b, a)| match (long, b, a) {
        (true, _, Some(a)) => px(*a) >= level,
        (false, Some(b), _) => px(*b) <= level,
        _ => false,
    });
    let Some(i0) = fill_at else {
        return Ok((
            None,
            Some("the entry day's ticks never reached the level: no trade".into()),
        ));
    };
    let (_, b0, a0) = q[i0];
    let entry_fill = if long {
        px(a0.unwrap_or_default()) + slip(t_e)
    } else {
        px(b0.unwrap_or_default()) - slip(t_e)
    };
    let initial_stop = if long { entry_fill - d } else { entry_fill + d };
    let mut stop = initial_stop;
    let mut best: Option<f64> = None;
    let better = |a: f64, b: f64| if long { a.max(b) } else { a.min(b) };
    let beyond = |r: f64, stop: f64| if long { r <= stop } else { r >= stop };
    let trail = |stop: f64, best: f64, atr: f64| {
        if long {
            stop.max(best - p.stop_atr * atr)
        } else {
            stop.min(best + p.stop_atr * atr)
        }
    };
    // Walk one day's quotes from index `from`, trailing on `atr`.
    let walk = |q: &[(i64, Option<i64>, Option<i64>)],
                from: usize,
                k: usize,
                atr: f64,
                stop: &mut f64,
                best: &mut Option<f64>|
     -> Option<(f64, f64, ExitReason)> {
        for (n, (_, b, a)) in q.iter().enumerate().skip(from) {
            let (Some(b), Some(a)) = (b, a) else { continue };
            let r = reference(k, *b, *a);
            if beyond(r, *stop) {
                let fill = if long {
                    px(*b) - slip(k)
                } else {
                    px(*a) + slip(k)
                };
                let why = if n == from && from == 0 {
                    ExitReason::GapThroughStop
                } else {
                    ExitReason::Stop
                };
                return Some((r, fill, why));
            }
            let nb = best.map_or(r, |x| better(x, r));
            *best = Some(nb);
            *stop = trail(*stop, nb, atr);
        }
        None
    };
    let mut exit: Option<(usize, f64, f64, ExitReason)> = None;
    if let Some((r, f, why)) = walk(&q, i0 + 1, t_e, atr_e, &mut stop, &mut best) {
        exit = Some((t_e, r, f, why));
    }
    let mut note = None;
    if exit.is_none() {
        if let (Some(b), Some(a)) = (best, ind.atr[t_e]) {
            stop = trail(stop, b, a);
        }
        let mut best_d = best.unwrap_or(if long { s.high[t_e] } else { s.low[t_e] });
        for u in (t_e + 1)..=s.last {
            let daily_exit = if long {
                s.open[u] <= stop || s.low[u] <= stop
            } else {
                s.open[u] >= stop || s.high[u] >= stop
            };
            if daily_exit {
                let qu = day_quotes(data, session, id, x.opens[u]).await?;
                if qu.is_empty() {
                    // No ticks that day: the daily-bar exit stands.
                    let gap = if long {
                        s.open[u] <= stop
                    } else {
                        s.open[u] >= stop
                    };
                    let r = if gap { s.open[u] } else { stop };
                    let f = if long {
                        r - s.to_bid[u] - slip(u)
                    } else {
                        r + s.to_ask[u] + slip(u)
                    };
                    note = Some(format!(
                        "no ticks on {}: the daily-bar exit is kept",
                        s.dates[u]
                    ));
                    exit = Some((
                        u,
                        r,
                        f,
                        if gap {
                            ExitReason::GapThroughStop
                        } else {
                            ExitReason::Stop
                        },
                    ));
                    break;
                }
                let atr_prev = ind.atr[u - 1].unwrap_or(atr_e);
                let mut b = Some(best_d);
                if let Some((r, f, why)) = walk(&qu, 0, u, atr_prev, &mut stop, &mut b) {
                    exit = Some((u, r, f, why));
                    break;
                }
                best_d = b.unwrap_or(best_d);
                if let Some(a) = ind.atr[u] {
                    stop = trail(stop, best_d, a);
                }
                continue;
            }
            best_d = better(best_d, if long { s.high[u] } else { s.low[u] });
            if let Some(a) = ind.atr[u] {
                stop = trail(stop, best_d, a);
            }
        }
    }
    let (t_x, exit_ref, exit_fill, exit_reason) = exit.unwrap_or_else(|| {
        let c = s.close[s.last];
        let f = if long {
            c - s.to_bid[s.last] - slip(s.last)
        } else {
            c + s.to_ask[s.last] + slip(s.last)
        };
        (s.last, c, f, ExitReason::PeriodEnd)
    });
    let gross_r = if long {
        (exit_fill - entry_fill) / d
    } else {
        (entry_fill - exit_fill) / d
    };
    let commission_r = (s.commission[t_e] + s.commission[t_x]) / d;
    let swaps = if long { &s.swap_long } else { &s.swap_short };
    let swap_r = swaps[t_e..t_x].iter().sum::<f64>() / d;
    Ok((
        Some(Trade {
            symbol: s.symbol.clone(),
            side,
            entry_index: t_e,
            entry_date: s.dates[t_e],
            level: Some(level),
            entry_ref: level,
            entry_fill,
            atr: atr_e,
            stop_distance: d,
            initial_stop,
            exit_index: t_x,
            exit_date: s.dates[t_x],
            exit_ref,
            exit_fill,
            exit_reason,
            nights: t_x - t_e,
            gross_r,
            commission_r,
            swap_r,
            net_r: gross_r - commission_r + swap_r,
            spread_estimated: s.spread_estimated[t_e] || s.spread_estimated[t_x],
        }),
        note,
    ))
}
