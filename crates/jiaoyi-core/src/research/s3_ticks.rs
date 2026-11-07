//! Bounded, single-session quote-path S3 research. Never account-performance evidence.
use super::{
    exit_replay as exits,
    s3_signal::{Bar, Request},
};
use crate::strategy3::{
    band, cost_reserve, sessions,
    targets::{self, Dir},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub const MAX_TICKS: usize = 200_000;
pub const MAX_PAGES: usize = 120;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    /// Broker cents of units, fixed for this scenario; NOT inferred account sizing.
    pub volume: i64,
    #[serde(
        default,
        skip_serializing_if = "super::opening_range::EntryRule::is_current"
    )]
    pub entry_rule: super::opening_range::EntryRule,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub digits: i32,
    pub min_volume: i64,
    pub step_volume: i64,
}
/// Chronological single-side broker update; price is in 1/100000 units.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tick {
    pub ts_ms: i64,
    pub price_e5: i64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Source {
    pub bars: Vec<Bar>,
    pub bids: Vec<Tick>,
    pub asks: Vec<Tick>,
    pub contract: Contract,
}
#[derive(Debug, Default, Serialize)]
pub struct Quality {
    pub bid_ticks: usize,
    pub ask_ticks: usize,
    pub snapshots: usize,
    pub missing_side: usize,
    pub stale_side: usize,
    pub crossed: usize,
    pub same_millisecond_groups: usize,
    pub long_intervals: usize,
}
#[derive(Debug, Serialize)]
pub struct Mark {
    pub at: DateTime<Utc>,
    pub reason: &'static str,
    pub decision_at: Option<DateTime<Utc>>,
    pub side: Option<Dir>,
    pub r: Option<f64>,
    pub cost: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct Trade {
    pub decision_at: DateTime<Utc>,
    pub entry: exits::Input,
    pub exit_comparison: exits::Comparison,
    pub trailing_scenario_net_r: Option<f64>,
    pub fixed_stop_scenario_net_r: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct Study {
    pub mode: &'static str,
    pub quality: Quality,
    pub marks: Vec<Mark>,
    pub trade: Option<Trade>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx_diagnostic: Option<super::fx_diagnostic::Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction_features: Option<super::class_prediction::Features>,
    pub notes: Vec<&'static str>,
}
pub fn validate_request(req: &Request) -> Result<(), String> {
    if req.to + Duration::seconds(5) > crate::seal::seal_instant() {
        return Err("five-second replay tail would cross the research seal".into());
    }
    let opts = req
        .tick_replay
        .as_ref()
        .ok_or("tick_replay options required")?;
    if opts.entry_rule.class_prediction() {
        return super::class_prediction::validate(req);
    }
    if opts.entry_rule == super::opening_range::EntryRule::FxSessionDiagnostic {
        return super::fx_diagnostic::validate(req);
    }
    let s = sessions::session_on(&req.rule(), req.from.date_naive())
        .ok_or("no session on this date")?;
    if req.from != s.open || req.to > s.close || req.to - req.from > Duration::days(1) {
        return Err("tick replay starts at the session open and ends no later than its close (maximum 24 hours)".into());
    }
    if opts.volume <= 0 || opts.volume > 1_000_000_000 || req.slippage_bps_per_side != 24.0 {
        return Err(
            "bounded positive fixed volume and current S3 24bp/side reserve required".into(),
        );
    }
    if !opts.entry_rule.is_current() && req.to != req.from + Duration::hours(4) {
        return Err("opening-range research requires a complete fixed four-hour episode".into());
    }
    Ok(())
}
#[derive(Clone, Copy)]
pub(super) struct Snapshot {
    pub(super) quote: exits::Quote,
    pub(super) fresh: bool,
}
pub(super) fn snapshots(req: &Request, src: &Source) -> Result<(Vec<Snapshot>, Quality), String> {
    if src.bids.len() + src.asks.len() > MAX_TICKS {
        return Err("tick budget exceeded".into());
    }
    let mut events = Vec::with_capacity(src.bids.len() + src.asks.len());
    for (side, ticks) in [(false, &src.bids), (true, &src.asks)] {
        let mut prev = None;
        for t in ticks {
            if prev.is_some_and(|p| t.ts_ms < p)
                || t.ts_ms < req.from.timestamp_millis()
                || t.ts_ms > req.to.timestamp_millis() + 5000
                || t.price_e5 <= 0
            {
                return Err("invalid, unordered or out-of-window tick".into());
            }
            let quantum = 10_i64.pow((5 - src.contract.digits) as u32);
            if t.price_e5 % quantum != 0 {
                return Err("tick is off the supplied price grid".into());
            }
            prev = Some(t.ts_ms);
            events.push((t.ts_ms, side, t.price_e5));
        }
    }
    // Stable sort retains within-side recorded order; no cross-side sequence is asserted.
    events.sort_by_key(|t| t.0);
    let mut q = Quality {
        bid_ticks: src.bids.len(),
        ask_ticks: src.asks.len(),
        ..Default::default()
    };
    let (mut bid, mut ask) = (None, None);
    let mut out = Vec::new();
    let mut i = 0;
    let mut previous = None;
    let mut pending_gap = false;
    while i < events.len() {
        let at = events[i].0;
        let start = i;
        while i < events.len() && events[i].0 == at {
            let e = events[i];
            if e.1 {
                ask = Some((at, e.2));
            } else {
                bid = Some((at, e.2));
            }
            i += 1;
        }
        let ambiguous = i - start > 1;
        q.same_millisecond_groups += usize::from(ambiguous);
        let long = previous.is_some_and(|p| at - p > 6000);
        previous = Some(at);
        q.long_intervals += usize::from(long);
        pending_gap |= long || ambiguous;
        let (Some((bt, b)), Some((atime, a))) = (bid, ask) else {
            q.missing_side += 1;
            pending_gap = true;
            continue;
        };
        let stale = at - bt > 6000 || at - atime > 6000;
        q.stale_side += usize::from(stale);
        if b > a {
            q.crossed += 1;
            pending_gap = true;
            continue;
        }
        out.push(Snapshot {
            quote: exits::Quote {
                at: DateTime::from_timestamp_millis(at).ok_or("timestamp overflow")?,
                bid: b as f64 / 100000.0,
                ask: a as f64 / 100000.0,
                gap_before: pending_gap || stale,
            },
            fresh: !stale,
        });
        pending_gap = false;
    }
    q.snapshots = out.len();
    Ok((out, q))
}
fn sign(side: Dir) -> f64 {
    if side == Dir::Buy {
        1.0
    } else {
        -1.0
    }
}
fn executable(q: &exits::Quote, side: Dir) -> f64 {
    if side == Dir::Buy {
        q.ask
    } else {
        q.bid
    }
}
pub(super) fn net(req: &Request, input: &exits::Input, o: &exits::Outcome) -> Option<f64> {
    let gross = o.scenario_gross_r?;
    let weighted_exit: f64 = o
        .fills
        .iter()
        .map(|f| f.price * f.volume as f64 / input.volume as f64)
        .sum();
    // Bid/ask fills already include spread. Do not charge it twice.
    Some(
        gross
            - (req.commission_round_trip
                + (input.fill + weighted_exit) * req.slippage_bps_per_side / 10000.0)
                / input.initial_r,
    )
}
pub fn study(req: &Request, src: &Source) -> Result<Study, String> {
    if matches!(req.symbol.as_str(), "SpotCrude" | "SpotBrent")
        && req
            .tick_replay
            .as_ref()
            .is_some_and(|o| o.entry_rule.is_current())
    {
        return Err(
            "oil current-S3 replay requires the offline preparation evidence adapter".into(),
        );
    }
    study_impl(req, src, false)
}

pub(super) fn study_impl(req: &Request, src: &Source, v7: bool) -> Result<Study, String> {
    req.validate(Utc::now())?;
    study_validated(req, src, v7)
}

// The offline seasonal adapter separately validates its exact four-symbol scope.
pub(super) fn study_validated(req: &Request, src: &Source, v7: bool) -> Result<Study, String> {
    study_with_gate(req, src, v7, None)
}

// Only the offline repair adapter supplies an alternative admission threshold.
// Costs and the existing target/exit calculation remain unchanged here.
pub(super) fn study_with_gate(
    req: &Request,
    src: &Source,
    v7: bool,
    dynamic_bars: Option<&[super::seasonal::M30]>,
) -> Result<Study, String> {
    study_with_policy(req, src, v7, dynamic_bars, None)
}

// Explicit local-file experiment only. Public request validation remains 24bp.
pub(super) fn study_with_native_cost_floor(
    req: &Request,
    src: &Source,
    bps: f64,
) -> Result<Study, String> {
    if ![24.0, 3.74, 2.0].contains(&bps)
        || !matches!(
            req.symbol.as_str(),
            "SpotCrude" | "SpotBrent" | "US500" | "XAUUSD"
        )
        || req
            .tick_replay
            .as_ref()
            .is_none_or(|o| !o.entry_rule.is_current())
    {
        return Err("unregistered offline native-cost experiment".into());
    }
    study_with_policy(req, src, true, None, Some(bps))
}

fn range_floor(r: f64, price: f64) -> bool {
    r.is_finite() && price.is_finite() && price > 0.0 && r * 10000.0 >= 48.0 * price
}

fn study_with_policy(
    req: &Request,
    src: &Source,
    v7: bool,
    dynamic_bars: Option<&[super::seasonal::M30]>,
    native_cost_bps: Option<f64>,
) -> Result<Study, String> {
    validate_request(req)?;
    let mut native_req = req.clone();
    if let Some(bps) = native_cost_bps {
        native_req.slippage_bps_per_side = bps;
    }
    let req = &native_req;
    let c = &src.contract;
    let volume = req.tick_replay.as_ref().unwrap().volume;
    if !(0..=5).contains(&c.digits)
        || c.min_volume <= 0
        || c.step_volume <= 0
        || volume < c.min_volume
        || volume % c.step_volume != 0
    {
        return Err("invalid contract or fixed scenario volume".into());
    }
    if req
        .tick_replay
        .as_ref()
        .unwrap()
        .entry_rule
        .class_prediction()
    {
        if !src.bars.is_empty() {
            return Err("class prediction requires tick-only source".into());
        }
        let (quotes, quality) = snapshots(req, src)?;
        return super::class_prediction::study(req, src, &quotes, quality);
    }
    // Share the M30 validator: no fabricated warm-up or duplicate bars.
    if req.tick_replay.as_ref().unwrap().entry_rule
        == super::opening_range::EntryRule::FxSessionDiagnostic
    {
        if !src.bars.is_empty() {
            return Err("FX session diagnostics require tick-only sources".into());
        }
        let (quotes, quality) = snapshots(req, src)?;
        return Ok(super::fx_diagnostic::study(req, &quotes, quality));
    }
    super::s3_signal::validated_bar_map(req, &src.bars)?;
    let (quotes, quality) = snapshots(req, src)?;
    if !req.tick_replay.as_ref().unwrap().entry_rule.is_current() {
        return super::opening_range::study(req, src, &quotes, quality);
    }
    let session = sessions::session_on(&req.rule(), req.from.date_naive()).unwrap();
    let bars: Vec<_> = src
        .bars
        .iter()
        .map(|b| band::M30 {
            open_time: b.at,
            open: b.open,
            close: b.close,
        })
        .collect();
    let table = band::sigma_table(&req.rule(), &session, &bars);
    let open = src
        .bars
        .iter()
        .find(|b| b.at == session.open)
        .map(|b| b.open);
    let mut result=Study{mode:"bounded_s3_tick_replay_v1",quality,marks:vec![],trade:None,fx_diagnostic:None,prediction_features:None,notes:vec![
        "DEVELOPMENT SCENARIO ONLY: no orders, no per-account financial PASS or goal progress claimed.",
        "Current shared S3 bands, mid-price mark decision, rounded R, cost reserve, durable session veto, one entry/session and target planner; current 30-second original-R trail cadence, no proposals in final 30 minutes.",
        "Fixed requested native volume and current contract snapshot; historical Kelly, margin, portfolio conflicts, ownership/restarts and historical contract changes are not reconstructed.",
        "Spread/commission inputs are scenarios, not certified historical medians. 24bp per side reserve retained. Spread is already in bid/ask fills; only commission and adverse slippage scenario are deducted afterwards.",
        "Entry at the next strictly later fresh quote within the same five-second mark window, with direction rechecked; no measured dispatch latency, rejection, liquidity or queue simulation.",
        "Exit comparison holds entry fixed: fixed stop versus S3 original-R trail. Old server bracket first, next-quote TP1/amend acknowledgement assumed, targets at level and gap stops at quote; path model is not a broker fill guarantee.",
        "Separate bid/ask history has no cross-side sequence. Same-millisecond updates form one end-of-millisecond snapshot; omitted intermediate ordering, stale/crossed quotes and long intervals are counted. Continuity remains unproven.",
        "Fetch includes five seconds beyond the requested boundary for a full-session close only. Partial-session studies censor open positions at the boundary, never invent a closing fill.",
    ]};
    let mut mark = session.open + Duration::minutes(30);
    let mut veto = false;
    let mut entered = false;
    while mark < req.to && mark < session.close - Duration::minutes(30) {
        let mut m = Mark {
            at: mark,
            reason: "no_fresh_quote_in_mark_window",
            decision_at: None,
            side: None,
            r: None,
            cost: None,
        };
        if veto {
            m.reason = "session_cost_veto";
        } else if entered {
            m.reason = "one_entry_already_consumed";
        } else if let Some(o) = open {
            match band::band_at(&table, &session, o, mark) {
                Err(_) => m.reason = "missing_band",
                Ok(b) => {
                    if let Some((idx, q)) = quotes.iter().enumerate().find(|(_, q)| {
                        q.fresh && q.quote.at >= mark && q.quote.at < mark + Duration::seconds(5)
                    }) {
                        m.decision_at = Some(q.quote.at);
                        m.reason = "inside_band_or_spread_too_wide";
                        if let Some(side) =
                            band::mark_decision(&b, Some(q.quote.bid), Some(q.quote.ask))
                        {
                            let price = executable(&q.quote, side);
                            let quantum = 10_i64.pow((5 - c.digits) as u32);
                            let e5 = (b.half_width * 100000.0).round() as i64;
                            let r = ((e5 + quantum / 2) / quantum).max(1) as f64 * quantum as f64
                                / 100000.0;
                            let cost = cost_reserve(
                                req.spread,
                                req.commission_round_trip,
                                req.slippage_bps_per_side,
                                price,
                            )?;
                            m.side = Some(side);
                            m.r = Some(r);
                            m.cost = Some(cost.total);
                            let decision_gate = if let Some(bars) = dynamic_bars {
                                let Some(atr) = super::three_year::atr(bars, mark) else {
                                    m.reason = "missing_atr14";
                                    result.marks.push(m);
                                    mark += Duration::minutes(30);
                                    continue;
                                };
                                (2.5 * (q.quote.ask - q.quote.bid)).max(0.15 * atr)
                            } else {
                                cost.total
                            };
                            if decision_gate >= r {
                                veto = true;
                                m.reason = if dynamic_bars.is_some() {
                                    "dynamic_range_session_veto"
                                } else {
                                    "cost_ge_r_session_veto"
                                };
                            } else if native_cost_bps.is_some() && !range_floor(r, price) {
                                // A low-volatility mark cannot enter. It does not create
                                // a new durable veto; the existing cost veto stays first.
                                m.reason = "range_below_48bp";
                            } else {
                                m.reason = "no_next_fresh_fill_quote_before_deadline";
                                if let Some((fi, fq)) =
                                    quotes.iter().enumerate().skip(idx + 1).find(|(_, f)| {
                                        f.fresh
                                            && f.quote.at > q.quote.at
                                            && f.quote.at < mark + Duration::seconds(5)
                                    })
                                {
                                    if band::mark_decision(
                                        &b,
                                        Some(fq.quote.bid),
                                        Some(fq.quote.ask),
                                    ) != Some(side)
                                    {
                                        m.reason = "direction_changed_before_simulated_fill";
                                    } else if v7
                                        && (if let Some(bars) = dynamic_bars {
                                            (2.5 * (fq.quote.ask - fq.quote.bid)).max(
                                                0.15 * super::three_year::atr(bars, mark)
                                                    .ok_or("missing dispatch ATR")?,
                                            )
                                        } else {
                                            cost_reserve(
                                                req.spread,
                                                req.commission_round_trip,
                                                req.slippage_bps_per_side,
                                                executable(&fq.quote, side),
                                            )?
                                            .total
                                        }) >= r
                                    {
                                        veto = true;
                                        m.reason = if dynamic_bars.is_some() {
                                            "dispatch_dynamic_range_session_veto"
                                        } else {
                                            "dispatch_cost_ge_r_session_veto"
                                        };
                                    } else if native_cost_bps.is_some()
                                        && !range_floor(r, executable(&fq.quote, side))
                                    {
                                        m.reason = "dispatch_range_below_48bp";
                                    } else {
                                        let p = targets::plan(&targets::Input {
                                            side,
                                            entry: price,
                                            original_stop: price - sign(side) * r,
                                            required_rr: targets::REQUIRED_RR,
                                            cost_reserve: cost.total,
                                            digits: c.digits,
                                            volume,
                                            min_volume: c.min_volume,
                                            step_volume: c.step_volume,
                                        })
                                        .map_err(str::to_string)?;
                                        let input = exits::Input {
                                            entered_at: fq.quote.at,
                                            fill: executable(&fq.quote, side),
                                            side: if side == Dir::Buy {
                                                exits::Side::Long
                                            } else {
                                                exits::Side::Short
                                            },
                                            initial_r: r,
                                            volume,
                                            tp1_close_volume: p.close_volume,
                                            tp1_distance: (p.trigger - price).abs(),
                                            broker_tp_distance: (p.broker_target - price).abs(),
                                            session_end: session.close,
                                            digits: c.digits,
                                            cadence: exits::TrailCadence::S3ThirtySeconds,
                                        };
                                        let mut engine = exits::ExitReplay::new(input.clone())
                                            .map_err(|e| e.to_string())?;
                                        let through = if req.to == session.close {
                                            req.to + Duration::seconds(5)
                                        } else {
                                            req.to
                                        };
                                        for f in quotes
                                            .iter()
                                            .skip(fi)
                                            .take_while(|f| f.quote.at <= through)
                                        {
                                            engine.push(f.quote).map_err(|e| {
                                                format!("invalid replay quote: {e:?}")
                                            })?;
                                        }
                                        let coverage = exits::Coverage {
                                            through,
                                            censored: true,
                                            ..exits::Coverage::recorded_path(through)
                                        };
                                        let comparison =
                                            engine.finish(coverage, exits::Costs::default());
                                        let trailing = net(req, &input, &comparison.candidate);
                                        let baseline = net(req, &input, &comparison.baseline);
                                        result.trade = Some(Trade {
                                            decision_at: q.quote.at,
                                            entry: input,
                                            exit_comparison: comparison,
                                            trailing_scenario_net_r: trailing,
                                            fixed_stop_scenario_net_r: baseline,
                                        });
                                        entered = true;
                                        m.reason = "simulated_entry";
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            m.reason = "missing_session_open";
        }
        result.marks.push(m);
        mark += Duration::minutes(30);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(symbol: &str, short: bool) -> (Request, Source) {
        let base = "2025-10-06T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let from = base + Duration::hours(if symbol == "NatGas" { 7 } else { 0 });
        let req = Request {
            account: 42,
            symbol: symbol.into(),
            from,
            to: from + Duration::hours(2),
            spread: 0.02,
            commission_round_trip: 0.0,
            slippage_bps_per_side: 24.0,
            tick_replay: Some(Options {
                volume: 4,
                entry_rule: Default::default(),
            }),
        };
        let mut src = Source {
            bars: vec![],
            bids: vec![],
            asks: vec![],
            contract: Contract {
                digits: 2,
                min_volume: 1,
                step_volume: 1,
            },
        };
        let mut at = req.history_from();
        while at < req.to {
            src.bars.push(Bar {
                at,
                open: 100.0,
                close: 101.0,
            });
            at += Duration::minutes(30);
        }
        let mid = if short { 96.0 } else { 104.0 };
        for (sec, price) in [
            (1800, mid),
            (1801, mid),
            (1810, mid + if short { -2.0 } else { 2.0 }),
            (1811, mid + if short { -2.0 } else { 2.0 }),
            (1841, mid),
        ] {
            pair(&mut src, from + Duration::seconds(sec), price);
        }
        (req, src)
    }
    fn pair(src: &mut Source, at: DateTime<Utc>, mid: f64) {
        src.bids.push(Tick {
            ts_ms: at.timestamp_millis(),
            price_e5: ((mid - 0.01) * 100000.0).round() as i64,
        });
        src.asks.push(Tick {
            ts_ms: at.timestamp_millis(),
            price_e5: ((mid + 0.01) * 100000.0).round() as i64,
        });
    }
    fn oil(short: bool) -> (Request, Source, super::super::production_s3::Preparation) {
        let (mut req, src) = fixture("NatGas", short);
        req.symbol = "SpotCrude".into();
        let evidence = super::super::production_s3::Preparation {
            source: "synthetic fixture, not market evidence".into(),
            available_at: req.from + Duration::minutes(30),
            opening_bar_tick_volume: 10,
            spread: req.spread,
            commission_round_trip: req.commission_round_trip,
        };
        (req, src, evidence)
    }
    #[test]
    fn prepared_oil_causal_marks_and_evidence() {
        use super::super::production_s3::study as prepared;
        let (mut req, mut src, mut e) = oil(false);
        req.to = req.from + Duration::hours(13);
        let out = prepared(&req, &src, &e).unwrap();
        assert_eq!(out.marks.len(), 24);
        let original = out.trade.unwrap().entry;
        pair(&mut src, req.from + Duration::hours(12), 150.0);
        assert_eq!(
            prepared(&req, &src, &e).unwrap().trade.unwrap().entry,
            original
        );
        e.opening_bar_tick_volume = 0;
        assert!(prepared(&req, &src, &e).is_err());
        e.opening_bar_tick_volume = 10;
        e.available_at += Duration::seconds(1);
        assert!(prepared(&req, &src, &e).is_err());
        assert!(study(&req, &src).is_err());
    }

    #[test]
    fn offline_native_cost_changes_admission_but_retains_floor_and_public_contract() {
        let (mut req, src, _) = oil(false);
        req.spread = 0.55;
        assert!(study_with_native_cost_floor(&req, &src, 24.0)
            .unwrap()
            .trade
            .is_none());
        let cheaper = study_with_native_cost_floor(&req, &src, 2.0).unwrap();
        let entry = cheaper.trade.unwrap().entry;
        assert!(range_floor(entry.initial_r, entry.fill));
        assert!(range_floor(0.48, 100.0));
        assert!(!range_floor(0.47999, 100.0));
        assert!(!range_floor(f64::NAN, 100.0));
        assert!(study_with_native_cost_floor(&req, &src, 1.0).is_err());
        req.slippage_bps_per_side = 2.0;
        assert!(validate_request(&req).is_err());
    }

    #[test]
    fn offline_dynamic_gate_uses_causal_price_units_and_rechecks_dispatch() {
        let (mut req, mut src, _) = oil(false);
        req.spread = 1.0; // Original reserve exceeds the fixture's 1R.
        let mark = req.from + Duration::minutes(30);
        let mut bars: Vec<_> = (0..15)
            .map(|i| super::super::seasonal::M30 {
                ts_ms: (mark - Duration::minutes((15 - i) * 30)).timestamp_millis(),
                open_e5: 10000000,
                high_e5: 10050000,
                low_e5: 9950000,
                close_e5: 10000000,
                volume: 1,
            })
            .collect();
        assert!(study_validated(&req, &src, true).unwrap().trade.is_none());
        let a = study_with_gate(&req, &src, true, Some(&bars)).unwrap();
        assert!(a.trade.is_some());
        assert!(a.marks[0].cost.unwrap() > a.marks[0].r.unwrap());
        // A bar opening at the decision is still incomplete and must not affect ATR.
        bars.push(super::super::seasonal::M30 {
            ts_ms: mark.timestamp_millis(),
            open_e5: 10000000,
            high_e5: 90000000,
            low_e5: 100000,
            close_e5: 80000000,
            volume: 1,
        });
        let b = study_with_gate(&req, &src, true, Some(&bars)).unwrap();
        assert_eq!(
            serde_json::to_value(&a.trade).unwrap(),
            serde_json::to_value(&b.trade).unwrap()
        );
        assert!(study_with_gate(&req, &src, true, Some(&[]))
            .unwrap()
            .trade
            .is_none());
        // The signal remains outside the band, but widening at dispatch fails G<R.
        src.asks[1].price_e5 = 10500000;
        src.bids[1].price_e5 = 10400000;
        let c = study_with_gate(&req, &src, true, Some(&bars)).unwrap();
        assert!(c.trade.is_none());
        assert_eq!(c.marks[0].reason, "dispatch_dynamic_range_session_veto");
    }
    #[test]
    fn prepared_oil_dispatch_cost_consumes_session_without_entry() {
        let (mut req, mut src, mut e) = oil(false);
        req.spread = 0.5;
        e.spread = req.spread;
        src.bids.clear();
        src.asks.clear();
        pair(&mut src, req.from + Duration::seconds(1800), 104.0);
        pair(&mut src, req.from + Duration::seconds(1801), 105.0);
        let out = super::super::production_s3::study(&req, &src, &e).unwrap();
        assert!(out.trade.is_none());
        assert_eq!(out.marks[0].reason, "dispatch_cost_ge_r_session_veto");
        assert_eq!(out.marks[1].reason, "session_cost_veto");
        assert!(out.marks[0].cost.unwrap() < out.marks[0].r.unwrap());
    }
    #[test]
    fn prepared_oil_full_exits_and_censoring_preserve_quality() {
        for short in [false, true] {
            let (req, mut src, e) = oil(short);
            let out = super::super::production_s3::study(&req, &src, &e).unwrap();
            assert!(out.quality.same_millisecond_groups > 0);
            let t = out.trade.unwrap();
            assert_eq!(t.exit_comparison.candidate.fills.len(), 2);
            assert!(t.exit_comparison.candidate.net_r.is_none());
            src.bids.truncate(2);
            src.asks.truncate(2);
            let t = super::super::production_s3::study(&req, &src, &e)
                .unwrap()
                .trade
                .unwrap();
            assert_eq!(t.exit_comparison.candidate.remaining_volume, 4);
            assert!(t.trailing_scenario_net_r.is_none());
        }
    }
    #[test]
    fn both_directions_partial_and_trailing_use_next_quote_original_r() {
        for (symbol, short) in [("BTCUSD", false), ("NatGas", true)] {
            let (req, src) = fixture(symbol, short);
            let out = study(&req, &src).unwrap();
            let t = out.trade.unwrap();
            assert_eq!(t.entry.entered_at, req.from + Duration::seconds(1801));
            assert!((t.entry.initial_r - 1.0).abs() < 1e-8);
            let x = &t.exit_comparison.candidate;
            assert_eq!(x.fills.len(), 2);
            assert_eq!(x.fills[0].reason, exits::ExitReason::Tp1MarketClose);
            assert_eq!(x.fills[0].at, req.from + Duration::seconds(1811));
            assert_eq!(x.fills[1].reason, exits::ExitReason::GapThroughStop);
            assert_eq!(x.remaining_volume, 0);
            assert!(x.net_r.is_none());
            assert!(t.trailing_scenario_net_r.is_some());
            assert!(out.quality.same_millisecond_groups > 0);
            assert_eq!(out.marks[1].reason, "one_entry_already_consumed");
        }
    }
    #[test]
    fn session_veto_is_durable_and_truncation_is_not_a_forced_exit() {
        let (mut req, src) = fixture("BTCUSD", false);
        req.spread = 2.0;
        let out = study(&req, &src).unwrap();
        assert!(out.trade.is_none());
        assert_eq!(out.marks[0].reason, "cost_ge_r_session_veto");
        assert_eq!(out.marks[1].reason, "session_cost_veto");
        req.spread = 0.02;
        let mut small = src.clone();
        small.bids.truncate(2);
        small.asks.truncate(2);
        let out = study(&req, &small).unwrap();
        let t = out.trade.unwrap();
        assert!(t.trailing_scenario_net_r.is_none());
        assert_eq!(t.exit_comparison.candidate.remaining_volume, 4);
        assert!(t
            .exit_comparison
            .candidate
            .data_issues
            .contains(&exits::DataIssue::Censored));
        // Future prices may change exits, never the prior entry.
        let mut future = src;
        pair(&mut future, req.from + Duration::seconds(1900), 120.0);
        assert_eq!(study(&req, &future).unwrap().trade.unwrap().entry, t.entry);
    }
    #[test]
    fn malformed_missing_stale_and_millisecond_sequences_are_explicit() {
        let (req, mut src) = fixture("BTCUSD", false);
        src.asks.clear();
        let out = study(&req, &src).unwrap();
        assert!(out.trade.is_none());
        assert!(out.quality.missing_side > 0);
        let (_, mut src) = fixture("BTCUSD", false);
        src.bids.swap(0, 1);
        assert!(study(&req, &src).unwrap_err().contains("unordered"));
        let (_, mut src) = fixture("BTCUSD", false);
        src.asks[0].price_e5 -= 10000;
        let out = study(&req, &src).unwrap();
        assert!(out.quality.crossed > 0);
        assert!(out.quality.long_intervals > 0);
        let mut invalid = req;
        invalid.from += Duration::minutes(30);
        assert!(invalid.validate(Utc::now()).is_err());
    }
    #[test]
    fn s3_trailing_uses_thirty_seconds_and_freezes_in_final_half_hour() {
        let start = "2025-10-06T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let input = exits::Input {
            entered_at: start,
            fill: 100.0,
            side: exits::Side::Long,
            initial_r: 1.0,
            volume: 1,
            tp1_close_volume: 1,
            tp1_distance: 20.0,
            broker_tp_distance: 20.0,
            session_end: start + Duration::minutes(31),
            digits: 2,
            cadence: exits::TrailCadence::S3ThirtySeconds,
        };
        let mut r = exits::ExitReplay::new(input).unwrap();
        for (sec, bid) in [
            (0, 100.1),
            (1, 100.1),
            (29, 100.2),
            (30, 100.3),
            (31, 100.3),
            (61, 101.0),
            (62, 101.0),
        ] {
            r.push(exits::Quote {
                at: start + Duration::seconds(sec),
                bid,
                ask: bid + 0.02,
                gap_before: false,
            })
            .unwrap();
        }
        let out = r.finish(
            exits::Coverage::recorded_path(start + Duration::seconds(62)),
            exits::Costs::default(),
        );
        assert_eq!(out.candidate.trail_proposals, 2);
        assert_eq!(out.candidate.trail_applied, 2);
        assert!((out.candidate.active_stop - 99.3).abs() < 1e-9);
        let (mut req, _) = fixture("BTCUSD", false);
        req.from = "2025-10-23T00:00:00Z".parse().unwrap();
        req.to = req.from + Duration::days(1);
        assert!(req.validate(Utc::now()).unwrap_err().contains("seal"));
    }
}
