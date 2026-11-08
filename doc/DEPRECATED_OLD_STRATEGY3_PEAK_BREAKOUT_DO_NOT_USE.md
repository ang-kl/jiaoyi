# DO NOT USE — retired Strategy 3 peak-breakout archive

> **WARNING — RETIRED LEGACY PEAK-BREAKOUT STRATEGY: DO NOT RESTORE OR MODIFY WITHOUT EXPLICIT HUMAN OWNER APPROVAL.**
>
> **Historical audit material only.** The owner retired this new-entry policy and its broad configured universe in directive №18,377. Code fences below preserve exact baseline source; they are not installation instructions or permission to reactivate it.

№18,377 §§1–6 · Archived 08-11-2026 20:10:28 SGT (+08:00) · by <Codex (GPT-6), effort: not exposed>.

## §1. What is retired and what remains protected

The archived production path bought an outside-band midpoint immediately at a 30-minute mark, or sold the corresponding lower-band break. It did not wait for a 50% retracement and did not enforce the new five-/twenty-session trend gate. New entries now belong to the owner-authorised macro-pullback production revision. The core whitelist is SpotCrude, SpotBrent, US500, NAS100, XAUUSD, XAGUSD, BTCUSD and ETHUSD; the requested eleven non-core symbols are inactive for new S3 entries.

The old broad-universe policy is retired. A dated description of “19 symbols” does not prove an account's effective historical universe: account/global database settings, earlier removals and broker resolution also governed it. This archive does not rewrite those settings receipts or claim that all nineteen symbols traded.

**Existing owned positions keep their recorded opening risk, target and exit policy.** Legacy TP1/runner and original-R Chandelier management may therefore remain executable solely for those positions. Removing that protection would contradict the owner's instruction. Retention of exit helpers, frozen research adapters or regression fixtures does not authorise new legacy breakout entries. Any future restoration or alteration of the retired entry strategy requires a new explicit human-owner instruction.

## §2. Empirical failure record and its limits

[PR #199](https://github.com/ang-kl/jiaoyi/pull/199) and the [260-position report](REBUILD_RFC_260DEALS_AND_RESERVATION_REPORT-18372-2026-11-08.md) reconcile **260 statement-matched positions, 36 net wins (13.85%), 224 non-winning positions and −$15,357.91 net P&L**. Gross fill-to-fill P&L was −$15,166.28; recorded commission and financing added $191.63 of drag. Spread/slippage may already be embedded in gross P&L.

**These 260 positions are not all one S3 strategy version.** The retained ledger contains 251 purely `jiaoyi-s3` labelled positions, seven unlabelled positions, one `jiaoyi-s1` position and one mixed S1/unlabelled position. It spans changed program versions. The aggregate loss is real; assigning every one of its 224 non-wins to this exact archived function would overstate the evidence. Registration hashes, entry keys and dated PR history are needed for version-specific attribution. The owner cited this loss record as motivation for replacement, not as a controlled causal experiment.

Separately, [Phase 2, PR #192](https://github.com/ang-kl/jiaoyi/pull/192), [report](REBUILD_PHASE2_PULLBACK_REPORT-18358-2026-11-08.md), evaluated an offline `arm_a_breakout_baseline` research arm on 35 qualified sessions. Under 24 bp/side stress costs it produced **0/3 positive forced-window closes, mean −1.064041R**. Only two trades had strict within-window closes (both losses); one needed the explicitly labelled window-end mark. This is a three-entry research result, not a replay of all 260 production positions, and its two-stage research exit differs from the legacy production TP1/runner exit archived here.

Neither result proves that inversion, pullback, a new trailing policy or the replacement model has an edge. Production authorisation and successful engineering tests do not establish a financial PASS or achievement of the 73% latest-20 goal.

## §3. Exact legacy formulas

| Component | Archived rule |
|---|---|
| Preparation | Causal M30 bars from a 45-calendar-day request; at each mark, the mean absolute session-open-to-mark return over the last 14 qualifying native sessions forms sigma. Exact missing-session rules are in the source below. |
| Band | `half_width = open × sigma`; `upper = open + half_width`; `lower = open − half_width`. |
| Decision | First eligible quote in the five-second window of a native 30-minute mark; both quote sides required. Midpoint above `upper` buys immediately; midpoint below `lower` sells immediately. No entry in the last 30 minutes. |
| Initial risk | `relative_stop = round_to_broker_price_grid(half_width)` with at least one tick; `R = relative_stop / 100000`. The exact nearest-grid integer implementation is in §9a. The broker applies the relative distance to its actual fill. |
| Cost reserve | `C = prepared_median_spread + round_trip_commission_per_unit + 2 × 24/10000 × executable_price`. The class function and exact calculation are preserved below. This is a reserve, not empirical complete round-trip cost calibration. |
| Cost veto | `C >= R` refuses the entry and stores a durable session veto. One recorded opening per account/symbol/session remains enforced. |
| TP1 | Outward-rounded distance `1.38 × R + C`. |
| TP1 allocation | The close fraction is `(R + C) / (rounded_TP1_distance + R)`, with close volume rounded up to the broker step and minimum. |
| TP2 | Runner target one further original `R` beyond TP1; broker holds TP2 for splittable positions. An unsplittable position has a whole-position broker TP1. |
| Chandelier | Best executable bid minus original `R` for longs; best executable ask plus original `R` for shorts. Only tightening by at least one grid tick; retain original opening-plan `R` across amendments/restarts. |
| Amendment throttle | At most one attempt per 30 seconds; the old last-30-minute amendment restriction remains part of this archived implementation. The throttle was not a 30-second lookback or a new definition of `R`. |
| Session exit | Close remaining owned quantity when the recorded native session has ended, preserving broker ownership/protection guards and retry semantics. |

## §4. Source identity

All code excerpts below were extracted byte-for-byte from commit [`25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f`](https://github.com/ang-kl/jiaoyi/tree/25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f), the pre-upgrade main baseline. The saved working copy of `strategy3.rs` was checked against that commit before extraction. They are historical snapshots, so references and comments inside them retain their original dates and terminology.

| Complete source at baseline | SHA256 |
|---|---|
| `crates/jiaoyi/src/strategy3.rs` | `8ae6e14c581126843585bfaba54f9c902ecf70e153aad25f5cbb08e501b7ba3c` |
| `crates/jiaoyi-core/src/strategy3/band.rs` | `483998e9e3a4206e1249b92d2804111c253f1a60db8f7902f00dc7a11a00f70c` |
| `crates/jiaoyi-core/src/strategy3/targets.rs` | `e373c96e321c9349522408249c4a545572756eb8431c10185e4948208d680c44` |
| `crates/jiaoyi-core/src/strategy3/mod.rs` | `8a195ad48abc52949b01c2c42e51a57da701d2639d37fe49db0abe815e8761f3` |
| `crates/jiaoyi/src/trading.rs` | `d25cbf2fde13291117f33b703883a8d508b801c120a5dd8f3f9510059a69c81a` |

## §5. Exact retired production entry dispatcher

Source: `crates/jiaoyi/src/strategy3.rs` at `25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f`. Excerpt SHA256: `16125f6d1ec81cac03c1c918bbeae6ec17d1bd5e4bf153f38e5ad98ca2d788ec`.

<details>
<summary>Read archived source — historical evidence only</summary>

```rust
    async fn entry(&mut self, symbol_id: i64, now: DateTime<Utc>) -> anyhow::Result<()> {
        let sym = self.syms[&symbol_id].clone();
        let name = sym.name.clone();
        let Some(rule) = sym.rule else {
            return Ok(());
        };
        let Some(s) = sessions::session_at(&rule, now) else {
            return Ok(());
        };
        if band::entry_window(&s, now).is_err() {
            return Ok(());
        }
        let Some(mark) = band::decision_mark(&s, now) else {
            return Ok(());
        };
        let dispatch_remaining = (mark + chrono::Duration::seconds(band::MARK_WINDOW_SECS) - now)
            .to_std()
            .unwrap_or_default();
        let dispatch_deadline = tokio::time::Instant::now() + dispatch_remaining;
        let dispatch_revision = self.session.connection().entry_revision(self.account_id());
        let (Some(bid), Some(ask)) = (sym.bid, sym.ask) else {
            return Ok(());
        };
        if !bid.is_finite() || !ask.is_finite() || bid > ask {
            return Ok(());
        }
        if sym.last_decision_mark.is_some_and(|at| at >= mark) {
            return Ok(());
        }
        // The session veto is durable, unlike the per-connection symbol cache.
        // A failed read must not consume this mark or permit a new entry.
        if cost_skip_stands(sym.cost_skip_session, s.date) {
            return Ok(());
        }
        if session_cost_veto(self.pool, self.account_id(), &name, s.date, &self.hash).await? {
            self.syms.get_mut(&symbol_id).unwrap().cost_skip_session = Some(s.date);
            tracing::info!(account = self.account_id(), symbol = %name,
                session_date = %s.date, prereg_hash = %self.hash,
                "strategy 3 restored durable session cost veto");
            return Ok(());
        }
        // Persist the first eligible quote before any preparation or entry
        // gate. An inside/no-band/refused decision must not be retried on a
        // later crossing within this mark, including after reconnect.
        let previous: Option<Option<String>> = jiaoyi_store::sqlx::query_scalar(
            "SELECT detail->>'decision_mark' FROM trading_decisions WHERE account_id=$1 AND symbol=$2 AND kind='s3_mark_decision' ORDER BY id DESC LIMIT 1")
            .bind(self.account_id()).bind(&name).fetch_optional(self.pool).await?;
        if let Some(at) = previous
            .flatten()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        {
            if at >= mark {
                self.syms.get_mut(&symbol_id).unwrap().last_decision_mark = Some(at);
                return Ok(());
            }
        }
        if sym.entered_session == Some(s.date) {
            self.syms.get_mut(&symbol_id).unwrap().last_decision_mark = Some(mark);
            return Ok(());
        }
        let prior = s3s::opens_in_session(self.pool, self.account_id(), symbol_id, s.date).await?;
        if session_already_entered(prior) {
            if let Some(x) = self.syms.get_mut(&symbol_id) {
                x.entered_session = Some(s.date);
                x.last_decision_mark = Some(mark);
            }
            self.skip(
                symbol_id,
                &name,
                "one entry already recorded for this symbol in this session",
                json!({"session_date": s.date, "opens": prior}),
                now,
            )
            .await;
            return Ok(());
        }
        ts::record_decision(self.pool, self.account_id(), Some(&name), "s3_mark_decision",
            "first eligible quote consumed this decision mark",
            &json!({"strategy":STRATEGY,"label":LABEL,"prereg_hash":self.hash,"decision_mark":mark,"decided_at":now,"bid":bid,"ask":ask}),
            self.d.version).await?;
        self.syms.get_mut(&symbol_id).unwrap().last_decision_mark = Some(mark);
        if sym
            .last_try
            .is_some_and(|t| now - t < chrono::Duration::from_std(ENTRY_RETRY).unwrap_or_default())
        {
            return Ok(());
        }
        if let Some(retry_at) = self.funding_retry_at.filter(|t| now < *t) {
            self.d
                .note(
                    Some(&name),
                    "funding_backoff",
                    "the broker rejected an entry for insufficient funds: account entry retries wait; exits continue",
                    json!({"retry_at": retry_at}),
                )
                .await;
            return Ok(());
        }
        self.prepare(symbol_id, s, now).await?;
        let mut sym = self.syms[&symbol_id].clone();
        sym.bid = Some(bid);
        sym.ask = Some(ask);
        let day = match &sym.day {
            Some(Ok(d)) if d.session.date == s.date => d.clone(),
            _ => return Ok(()),
        };
        let Ok(open) = day.open.clone() else {
            return Ok(());
        };
        let (side, b) = match signal(&day.table, &s, open, now, sym.bid, sym.ask) {
            Ok(Some(x)) => x,
            Ok(None) => return Ok(()),
            Err(why) => {
                self.d.note(Some(&name), "no_band", &why, json!({})).await;
                return Ok(());
            }
        };
        let entry_price = match side {
            Dir::Buy => sym.ask,
            Dir::Sell => sym.bid,
        }
        .unwrap_or(f64::NAN);
        let mut detail = json!({
            "session_date": s.date, "decided_at": now, "bid": sym.bid, "ask": sym.ask,
            "side": side, "band": b, "open": open, "sigma": b.sigma,
        });
        let Some(limits) = sym.limits else {
            self.skip(
                symbol_id,
                &name,
                "the symbol's volume rules are not stored",
                detail,
                now,
            )
            .await;
            return Ok(());
        };
        // Keep the actual entry-time grid and UTC session for later replay;
        // today's symbol metadata or calendar must not replace this snapshot.
        detail["volume_limits"] = json!(limits);
        detail["session"] = json!(s);
        let spread = match &day.spread {
            Ok((m, _)) => *m,
            Err(why) => {
                self.skip(symbol_id, &name, why, detail, now).await;
                return Ok(());
            }
        };
        let Some(rate) = self.rate(symbol_id) else {
            self.skip(
                symbol_id,
                &name,
                "no broker quote→deposit rate for this symbol yet",
                detail,
                now,
            )
            .await;
            return Ok(());
        };
        if self
            .commission_read
            .is_none_or(|t| t.elapsed() >= COMMISSION_REFRESH)
        {
            self.read_commission().await;
        }
        let commission_rt = match &self.commission {
            Ok(book) => {
                let (per_unit, label) = book
                    .per_class
                    .get(sym.class.as_str())
                    .cloned()
                    .unwrap_or((book.overall_median, "estimated: the overall median".into()));
                detail["commission_label"] = json!(label);
                2.0 * per_unit / rate
            }
            Err(why) => {
                self.skip(
                    symbol_id,
                    &name,
                    &format!("commission not measured: {why}"),
                    detail,
                    now,
                )
                .await;
                return Ok(());
            }
        };
        let cost = match cost_reserve(
            spread,
            commission_rt,
            slippage_bps_per_side(sym.class),
            entry_price,
        ) {
            Ok(c) => c,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        let rel = relative_stop(b.half_width, limits.digits);
        let r_price = rel as f64 / 100_000.0;
        // v3 cost gate (27-10-2026). The statements measured median C ≈ 2.2R,
        // so TP1 sat near 5R and a no-drift process hit it ~16% of the time.
        // Skip when the reserve is at least the stop. This does not change Q.
        if !(r_price > 0.0) || cost.total >= r_price {
            detail["strategy"] = json!(STRATEGY);
            detail["label"] = json!(LABEL);
            detail["prereg_hash"] = json!(self.hash);
            detail["skip_code"] = json!("cost_ge_r");
            let reason = format!(
                "cost reserve C ({:.6}) >= R ({:.6}): barrier too far for a no-drift process; skipped",
                cost.total, r_price
            );
            // This is execution state, not best-effort reporting. Persist it
            // before the cache suppresses subsequent quotes for this session.
            ts::record_decision(
                self.pool,
                self.account_id(),
                Some(&name),
                "s3_skip",
                &reason,
                &detail,
                self.d.version,
            )
            .await?;
            if let Some(x) = self.syms.get_mut(&symbol_id) {
                x.cost_skip_session = Some(s.date);
                x.last_try = Some(now);
            }
            tracing::info!(account = self.account_id(), symbol = %name,
                kind = "skip", reason, prereg_hash = %self.hash, "strategy 3 decision");
            return Ok(());
        }
        let e = Entry {
            side,
            band: b,
            entry_price,
            relative_stop: rel,
            r: rel as f64 / 100_000.0,
            cost,
        };
        detail["entry"] = json!(e);
        detail["quote_to_deposit"] = json!(rate);
        let r = rules(self.pool, self.account_id(), self.env).await?;
        self.d.version = Some(r.version);
        // Claude · № 17,116 25-Oct (ordered: owner "continue"; C3 per №17,115):
        // the owner's pause ("only human can pause trade"): no new entry. TP1,
        // the session end and the broker's stop and take-profit are untouched.
        if r.paused {
            if let Some(s) = self.syms.get_mut(&symbol_id) {
                s.last_try = Some(now);
            }
            self.d
                .note(
                    Some(&name),
                    "paused",
                    "Strategy 3 is paused by the owner on this account (trading.s3_paused): no new entry; open positions keep their exits",
                    detail,
                )
                .await;
            return Ok(());
        }
        // Claude · № 17,020 25-Oct (Codex P1 on #15): a live entry is one
        // broker minimum, admitted account-wide (both strategies' loops) from
        // fresh broker state; the account's lock is held through the send.
        let _admission = if r.live {
            let g = crate::admission::lock(self.account_id()).await;
            if let Err(why) = crate::admission::live_minimum(self.pool, self.session).await {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
            Some(g)
        } else {
            None
        };
        let equity = match fresh_equity(self.session).await {
            Ok(x) => x,
            Err(why) => {
                self.skip(
                    symbol_id,
                    &name,
                    &format!("fresh equity unreadable: {why}"),
                    detail,
                    now,
                )
                .await;
                return Ok(());
            }
        };
        // Live: the fresh account-wide admission above is the count.
        let open_programs = if r.live {
            0
        } else {
            self.program_positions + self.pending.len()
        };
        // Kelly (№ 17,588–17,589, 28-10-2026). A ceiling and a skip in front of
        // size(). It does not write risk_pct, the live minimum, or the budget.
        // Under 20 closes on this hash and symbol: not an input. A non-positive
        // fraction skips. A positive fraction is recorded and does not raise size.
        // Grok 4.7. Other models: do not plug in the goal, and do not double
        // after a loss.
        let keys =
            s3s::open_keys_for_hash(self.pool, self.account_id(), symbol_id, &self.hash).await?;
        let rs = kelly_sample(self.pool, self.account_id(), &keys).await?;
        let queried = rs.is_some();
        let kelly = jiaoyi_core::strategy3::kelly::read(rs.as_deref().unwrap_or_default());
        detail["kelly"] = json!({
            "n": queried.then_some(kelly.n),
            "wins": queried.then_some(kelly.wins),
            "history_queried": queried,
            "history_status": if queried { "queried" } else { "below_minimum_possible_positions" },
            "eligible_open_keys": keys.len(),
            "p": kelly.p,
            "b": kelly.b,
            "f_star": kelly.f_star,
            "quarter": kelly.quarter,
            "min_closes": jiaoyi_core::strategy3::kelly::MIN_CLOSES,
            "writes_size": false,
        });
        if jiaoyi_core::strategy3::kelly::skip(&kelly) {
            self.skip(
                symbol_id,
                &name,
                &format!(
                    "Kelly f* ({:.4}) is not positive on {} closes under this hash: skipped",
                    kelly.f_star.unwrap_or(f64::NAN),
                    kelly.n
                ),
                detail,
                now,
            )
            .await;
            return Ok(());
        }
        let sized = match size(&r, equity, rel, rate, &limits, open_programs) {
            Ok(x) => x,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        detail["sizing"] = json!(sized);
        let order = match plan_order(&e, &limits, sized.volume) {
            Ok(o) => o,
            Err(why) => {
                self.skip(symbol_id, &name, &why, detail, now).await;
                return Ok(());
            }
        };
        detail["order"] = json!(order);
        let budget = self
            .open_risk()
            .and_then(|held| within_budget(held, sized.risk, equity, r.open_risk_pct));
        if let Err(why) = budget {
            self.skip(symbol_id, &name, &why, detail, now).await;
            return Ok(());
        }
        detail["dispatch_policy"] = json!(jiaoyi_ctrader::entry_guard::POLICY);
        detail["dispatch_deadline"] =
            json!(mark + chrono::Duration::seconds(band::MARK_WINDOW_SECS));
        let entry_guard = jiaoyi_ctrader::entry_guard::EntryGuard {
            connection_generation: self.session.connection().entry_generation(),
            account_id: self.account_id(),
            symbol_id,
            side: dir_side(side),
            signal: jiaoyi_ctrader::entry_guard::Signal::Mark { band: b },
            cost: Some(jiaoyi_ctrader::entry_guard::Cost {
                prepared_spread: spread,
                commission_round_trip: commission_rt,
                slippage_bps_per_side: slippage_bps_per_side(sym.class),
                original_r: r_price,
            }),
            expires: dispatch_deadline,
            account_revision: dispatch_revision,
        };
        let account_id = self.account_id();
        let n = s3s::attempts_in_session(self.pool, account_id, symbol_id, s.date).await?;
        let key = format!("s3:{account_id}:{symbol_id}:{}:{}", s.date, n + 1);
        if let Some(x) = self.syms.get_mut(&symbol_id) {
            x.last_try = Some(now);
        }
        let ledger = S3Ledger {
            inner: PgLedger {
                pool: self.pool,
                settings_version: Some(r.version),
            },
            order: s3s::OrderRow {
                account_id,
                entry_key: key.clone(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: "open".into(),
                symbol: name.clone(),
                symbol_id,
                position_id: None,
                side: Some(dir_side(side).as_str().into()),
                session_date: Some(s.date),
                decided_at: now,
                bid: sym.bid,
                ask: sym.ask,
                volume: Some(sized.volume),
                detail: detail.clone(),
            },
            recorded_at: Mutex::new(None),
        };
        let cmd = Command::Open {
            symbol_id,
            side: dir_side(side),
            volume: sized.volume,
            relative_stop: rel,
            relative_take_profit: Some(order.relative_take_profit),
            limits,
            quote_to_deposit: Some(rate),
            label: LABEL.into(),
        };
        let policy = Policy {
            edge: Ok(()),
            max_risk: sized.max_risk,
        };
        match gate::send_entry(
            self.session,
            self.allowlist,
            &ledger,
            &policy,
            cmd,
            &key,
            Duration::from_secs(r.order_wait_secs),
            &entry_guard,
        )
        .await
        {
            Ok(sent) => {
                self.record_fill(
                    &ledger,
                    &key,
                    "open",
                    &name,
                    None,
                    dir_side(side),
                    Some(entry_price),
                    sym.bid,
                    sym.ask,
                    sized.volume,
                    &sent,
                )
                .await?;
                // Any fill, whatever the last answer (several deals, or part
                // of the order before a rejection or no answer), is read from
                // the broker at once: its aggregate volume, entry, stop and
                // take-profit, with its recorded plan. An event carries one
                // deal, not the position (Codex P1 on #15; Strategy 1 does
                // the same).
                // Claude · № 17,060 25-Oct (ordered: owner "merge when green
                // and deploy"; Codex round 3 on #15)
                if matches!(&sent.outcome, Outcome::Rejected { code: Some(code), .. } if code == "NOT_ENOUGH_MONEY")
                {
                    self.funding_retry_at =
                        Some(now + chrono::Duration::from_std(ENTRY_RETRY).unwrap_or_default());
                }
                if matches!(sent.outcome, Outcome::Filled { .. }) || partly_filled(&sent) {
                    self.reconcile(Utc::now()).await?;
                } else if matches!(sent.outcome, Outcome::Unconfirmed) {
                    self.pending.push(Pending {
                        entry_key: key.clone(),
                        symbol_id,
                        risk: sized.risk,
                        at: now,
                    });
                }
                self.d
                    .note(
                        Some(&name),
                        "sent",
                        &format!(
                            "{STRATEGY} {} {} cents ({}): {:?}",
                            dir_side(side).as_str(),
                            sized.volume,
                            sized.mode,
                            sent.outcome
                        ),
                        json!({"entry_key": key, "relative_stop": rel,
                               "relative_take_profit": order.relative_take_profit}),
                    )
                    .await;
            }
            Err(SendError::Refused(x)) => {
                self.d.note(Some(&name), "refused", &x.reason, detail).await;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
```

</details>

## §6. Exact production signal and bracket planner

Source: `crates/jiaoyi/src/strategy3.rs` at `25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f`. Excerpt SHA256: `a23263d53abc844f6ece92ea94cbf880487b3500c46bc73895f0367fa7f8d064`.

<details>
<summary>Read archived source — historical evidence only</summary>

```rust
pub fn signal(
    table: &SigmaTable,
    session: &Session,
    open: f64,
    now: DateTime<Utc>,
    bid: Option<f64>,
    ask: Option<f64>,
) -> Result<Option<(Dir, Band)>, String> {
    band::entry_window(session, now)?;
    let Some(mark) = band::decision_mark(session, now) else {
        return Ok(None);
    };
    let b = band::band_at(table, session, open, mark)?;
    Ok(band::mark_decision(&b, bid, ask).map(|d| (d, b)))
}

pub fn distance_e5(distance: f64, digits: i32) -> i64 {
    let ticks = (distance.abs() * 10f64.powi(digits)).round() as i64;
    ticks * 10_i64.pow((5 - digits.clamp(0, 5)) as u32)
}

pub fn plan_order(e: &Entry, limits: &VolumeLimits, volume: i64) -> Result<Order, String> {
    let r = e.relative_stop as f64 / 100_000.0;
    let stop = match e.side {
        Dir::Buy => e.entry_price - r,
        Dir::Sell => e.entry_price + r,
    };
    let plan = targets::plan(&targets::Input {
        side: e.side,
        entry: e.entry_price,
        original_stop: stop,
        required_rr: targets::REQUIRED_RR,
        cost_reserve: e.cost.total,
        digits: limits.digits,
        volume,
        min_volume: limits.min,
        step_volume: limits.step,
    })
    .map_err(|why| format!("bot-trade {}: {why}", targets::POLICY))?;
    let tp1 = (plan.trigger - e.entry_price).abs();
    let tp2 = (plan.runner_target - e.entry_price).abs();
    let broker = (plan.broker_target - e.entry_price).abs();
    Ok(Order {
        plan,
        relative_take_profit: distance_e5(broker, limits.digits),
        tp1_distance: tp1,
        tp2_distance: tp2,
    })
}
```

</details>

## §7. Exact original-R Chandelier helper and amendment path

Source: `crates/jiaoyi/src/strategy3.rs` at `25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f`. Excerpt SHA256: `b59443e693e5bddea2d03db06f1a3d9254eae163b6f599409b3764968ee0a990`.

<details>
<summary>Read archived source — historical evidence only</summary>

```rust
pub fn chandelier_stop(side: Side, best: f64, r: f64) -> Option<f64> {
    if !best.is_finite() || best <= 0.0 || !r.is_finite() || r <= 0.0 {
        return None;
    }
    let stop = match side {
        Side::Buy => best - r,
        Side::Sell => best + r,
    };
    (stop.is_finite() && stop > 0.0).then_some(stop)
}

fn trail_price(side: Side, best: f64, r: f64, current: f64, digits: i32) -> Option<f64> {
    let factor = 10f64.powi(digits);
    let grid = |price: f64| {
        let ticks = targets::price_ticks(price, digits)?;
        // Same float-residue convention as the original target planner.
        (price > 0.0 && (price * factor - ticks as f64).abs() < 1e-6).then_some(ticks)
    };
    let (best, r, current) = (grid(best)?, grid(r)?, grid(current)?);
    if r <= 0 {
        return None;
    }
    let stop = match side {
        Side::Buy => best.checked_sub(r)?,
        Side::Sell => best.checked_add(r)?,
    };
    let improves = match side {
        Side::Buy => stop > current,
        Side::Sell => stop < current,
    };
    (stop > 0 && improves).then_some(stop as f64 / factor)
}

    async fn trail(
        &mut self,
        position_id: i64,
        bid: Option<f64>,
        ask: Option<f64>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let Some(o) = self.own.get(&position_id).cloned() else {
            return Ok(());
        };
        let (Some(_), Some(current_stop)) = (o.entry, o.stop) else {
            return Ok(());
        };
        // v5 uses the original opening-plan R, also restored on reconcile.
        // The broker's current stop changes after each amend and can reach
        // entry; it cannot establish the original distance when the plan is
        // missing. Keep the existing protection in that case.
        let Some(r) = o.plan_basis.map(|basis| basis.r) else {
            return Ok(());
        };
        if !r.is_finite() || r <= 0.0 {
            return Ok(());
        }
        let price = match o.side {
            Side::Buy => bid,
            Side::Sell => ask,
        };
        let Some(p) = price.filter(|x| x.is_finite() && *x > 0.0) else {
            return Ok(());
        };
        let best = match (o.best, o.side) {
            (Some(b), Side::Buy) => b.max(p),
            (Some(b), Side::Sell) => b.min(p),
            (None, _) => p,
        };
        if let Some(x) = self.own.get_mut(&position_id) {
            x.best = Some(best);
            if let Some(entry) = x.entry {
                let (mae, mfe) = update_excursions(x.side, entry, p, x.mae, x.mfe);
                x.mae = mae;
                x.mfe = mfe;
            }
        }
        let Some(digits) = self
            .syms
            .get(&o.symbol_id)
            .and_then(|s| s.limits)
            .map(|l| l.digits)
        else {
            return Ok(());
        };
        let Some(new_stop) = trail_price(o.side, best, r, current_stop, digits) else {
            return Ok(());
        };
        if o.last_amend
            .is_some_and(|t| now.signed_duration_since(t).num_seconds() < TRAIL_AMEND_SECS)
        {
            return Ok(());
        }
        // No amend in the session's last 30 minutes.
        if let Some(sym) = self.syms.get(&o.symbol_id) {
            if let Some(session) = sym.rule.and_then(|rule| sessions::session_at(&rule, now)) {
                if session.close - now < chrono::Duration::minutes(30) {
                    return Ok(());
                }
            }
        }
        let account_id = self.account_id();
        let name = self
            .syms
            .get(&o.symbol_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let ticks = (new_stop * 100_000.0).round() as i64;
        let key = format!("s3-trail:{account_id}:{}:{ticks}", o.position_id);
        let rules = rules(self.pool, account_id, self.env).await?;
        // kind 'trail' is allowed by migration 0019. Before that the check
        // rejected the row after the intent was stored, so the frame was
        // never written. Grok 4.7 · № 17,655.
        let order = s3s::OrderRow {
            account_id,
            entry_key: key.clone(),
            label: LABEL.into(),
            prereg_hash: self.hash.clone(),
            kind: "trail".into(),
            symbol: name.clone(),
            symbol_id: o.symbol_id,
            position_id: Some(o.position_id),
            side: Some(o.side.as_str().into()),
            session_date: o.session_date,
            decided_at: now,
            bid,
            ask,
            volume: Some(o.volume),
            detail: json!({"strategy": STRATEGY, "from_stop": current_stop, "best": best,
                "r": r, "new_stop": new_stop, "open_entry_key": o.entry_key}),
        };
        let ledger = S3Ledger {
            inner: PgLedger {
                pool: self.pool,
                settings_version: Some(rules.version),
            },
            order,
            recorded_at: Mutex::new(None),
        };
        let policy = Policy {
            edge: Ok(()),
            max_risk: f64::INFINITY,
        };
        let cmd = Command::AmendStop {
            position_id: o.position_id,
            side: o.side,
            current_stop: Some(current_stop),
            new_stop,
        };
        match gate::send(
            self.session,
            self.allowlist,
            &ledger,
            &policy,
            cmd,
            &key,
            Duration::from_secs(rules.order_wait_secs),
        )
        .await
        {
            Ok(sent) => {
                if let Some(x) = self.own.get_mut(&position_id) {
                    x.last_amend = Some(now);
                    if matches!(sent.outcome, Outcome::Amended { .. }) {
                        x.stop = Some(new_stop);
                    }
                }
                self.record_fill(
                    &ledger,
                    &key,
                    "trail",
                    &name,
                    Some(o.position_id),
                    o.side,
                    Some(new_stop),
                    bid,
                    ask,
                    o.volume,
                    &sent,
                )
                .await?;
                self.d
                    .note(
                        Some(&name),
                        "trail",
                        &format!("stop to {new_stop}: {:?}", sent.outcome),
                        json!({"position_id": o.position_id, "from_stop": current_stop, "best": best,
                            "entry_key": key}),
                    )
                    .await;
            }
            Err(e @ (SendError::Evidence(_) | SendError::Authority(_))) => return Err(e.into()),
            Err(e) => {
                if let Some(x) = self.own.get_mut(&position_id) {
                    x.last_amend = Some(now);
                }
                self.d
                    .note(
                        Some(&name),
                        "trail_refused",
                        &e.to_string(),
                        json!({"position_id": o.position_id, "entry_key": key}),
                    )
                    .await;
            }
        }
        Ok(())
    }
```

</details>

## §8. Exact band, session-mark and sigma definitions

Source: `crates/jiaoyi-core/src/strategy3/band.rs` at `25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f`. Excerpt SHA256: `e03ed74d538b8b71f90c8b79307552746f71a36d208d2f698a0c0404326434bc`.

<details>
<summary>Read archived source — historical evidence only</summary>

```rust
//! Strategy 3's noise band (owner-approved rules, 25-10-2026; after
//! Zarattini, Aziz & Barbon 2024, SFI RP 24-97).
//!
//! - The session's **open** is its first M30 trendbar's open, the bar that
//!   starts at the session open.
//! - At each 30-minute mark τ after the open, **σ(τ)** is the mean, over the
//!   last [`LOOKBACK`] sessions, of |price at τ ÷ that session's open − 1|.
//!   The price at τ is the close of the M30 bar that ends at τ.
//! - The band is open × (1 ± σ).
//!
//! **Refinement, 27-10-2026 (v3).** The 25 Oct registration entered on the
//! first tick beyond the band. The statements of that book printed the
//! barrier rate of a far target, not an edge. v3 decides only at a
//! 30-minute mark, on the mid, which is the paper's check. [`triggered`] is
//! kept for the recorded tick rule and its tests. Live entry uses
//! [`at_decision_mark`] and [`mark_decision`]. See
//! `docs/preregistration-2026-10-27-strategy3-v3.md`.
//!
//! Readings where the text leaves a choice (self-derived, recorded with
//! every decision):
//! - **Missing bars.** A past session counts only if it has the bar that
//!   starts at its open. At a mark it counts only if it has the bar that
//!   ends at that mark. σ(τ) takes the last 14 sessions that do, looking back
//!   over at most [`MAX_SESSIONS_BACK`] sessions. Fewer than 14 means no band
//!   at τ, with the reason.
//! - **Between marks.** At an instant between two marks, σ is interpolated
//!   linearly between them. v3 does not decide between marks.
//! - **Before the first mark.** There is no band before open + 30 minutes.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::sessions::{sessions_before, Session, SessionRule};

/// Sessions σ is averaged over (rules: "the last 14 sessions").
pub const LOOKBACK: usize = 14;
/// How far back sessions are looked for, to find 14 with a given bar.
pub const MAX_SESSIONS_BACK: usize = 30;
/// The band is checked from this many minutes after the open, every as many.
pub const MARK_MINUTES: i64 = 30;

/// An M30 trendbar: its open time and its open and close prices.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct M30 {
    pub open_time: DateTime<Utc>,
    pub open: f64,
    pub close: f64,
}

/// σ at each 30-minute mark of one session, from the sessions before it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SigmaTable {
    /// Minutes after the open → σ, or why there is none.
    pub marks: BTreeMap<i64, Result<f64, String>>,
    /// The past sessions that had a first bar (newest first): their dates.
    pub sessions_used: Vec<chrono::NaiveDate>,
}

/// The bar starting at `at`, if any.
fn bar_at(bars: &BTreeMap<DateTime<Utc>, M30>, at: DateTime<Utc>) -> Option<&M30> {
    bars.get(&at)
}

/// σ at every 30-minute mark of `today`, from M30 bars of the sessions
/// before it (`bars` may hold any span; only the bars that sit on the
/// sessions' marks are read).
pub fn sigma_table(rule: &SessionRule, today: &Session, bars: &[M30]) -> SigmaTable {
    let by_time: BTreeMap<DateTime<Utc>, M30> = bars.iter().map(|b| (b.open_time, *b)).collect();
    // Past sessions with a first bar, newest first.
    let past: Vec<(Session, f64)> = sessions_before(rule, today.open, MAX_SESSIONS_BACK)
        .into_iter()
        .filter_map(|s| {
            let o = bar_at(&by_time, s.open)?.open;
            (o.is_finite() && o > 0.0).then_some((s, o))
        })
        .collect();
    let length = (today.close - today.open).num_minutes();
    let mut marks = BTreeMap::new();
    let mut tau = MARK_MINUTES;
    while tau <= length {
        let mut moves = Vec::new();
        for (s, o) in &past {
            if moves.len() == LOOKBACK {
                break;
            }
            // The bar that ends at τ starts 30 minutes earlier.
            let start = s.open + Duration::minutes(tau - MARK_MINUTES);
            if start + Duration::minutes(MARK_MINUTES) > s.close {
                continue;
            }
            if let Some(b) = bar_at(&by_time, start) {
                if b.close.is_finite() && b.close > 0.0 {
                    moves.push((b.close / o - 1.0).abs());
                }
            }
        }
        let v = if moves.len() == LOOKBACK {
            Ok(moves.iter().sum::<f64>() / LOOKBACK as f64)
        } else {
            Err(format!(
                "{} of the last {} sessions have a price at {} minutes after the open; {LOOKBACK} needed",
                moves.len(),
                past.len(),
                tau
            ))
        };
        marks.insert(tau, v);
        tau += MARK_MINUTES;
    }
    SigmaTable {
        marks,
        sessions_used: past.iter().map(|(s, _)| s.date).collect(),
    }
}

/// The band at one instant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Band {
    /// The session's open (its first M30 bar's open).
    pub open: f64,
    pub sigma: f64,
    pub lower: f64,
    pub upper: f64,
    /// The half-width, open × σ: the trade's R.
    pub half_width: f64,
    /// Minutes after the session open.
    pub minutes: f64,
}

/// σ at `minutes` after the open: a mark's value, or the linear
/// interpolation between the two marks around it.
pub fn sigma_at(table: &SigmaTable, minutes: f64) -> Result<f64, String> {
    if minutes.is_nan() || minutes < MARK_MINUTES as f64 {
        return Err(format!(
            "no band before {MARK_MINUTES} minutes after the open ({minutes:.1})"
        ));
    }
    let lo = (minutes / MARK_MINUTES as f64).floor() as i64 * MARK_MINUTES;
    let get = |m: i64| match table.marks.get(&m) {
        Some(Ok(v)) => Ok(*v),
        Some(Err(why)) => Err(why.clone()),
        None => Err(format!("no mark at {m} minutes in this session")),
    };
    let a = get(lo)?;
    let frac = (minutes - lo as f64) / MARK_MINUTES as f64;
    if frac <= 0.0 {
        return Ok(a);
    }
    let b = get(lo + MARK_MINUTES)?;
    Ok(a + (b - a) * frac)
}

/// The band at `at` in `today`, around `open`.
pub fn band_at(
    table: &SigmaTable,
    today: &Session,
    open: f64,
    at: DateTime<Utc>,
) -> Result<Band, String> {
    if !(open.is_finite() && open > 0.0) {
        return Err("no session open price".into());
    }
    let minutes = (at - today.open).num_milliseconds() as f64 / 60_000.0;
    let sigma = sigma_at(table, minutes)?;
    let half_width = open * sigma;
    Ok(Band {
        open,
        sigma,
        lower: open - half_width,
        upper: open + half_width,
        half_width,
        minutes,
    })
}

/// The side a quote triggers against the band at its instant: an ask above
/// the upper band buys, a bid below the lower band sells. A quote doing both
/// (a spread wider than the band) triggers nothing; a quote on the band is
/// inside it.
///
/// **Not the live entry.** This is the 25 Oct tick rule, kept so its tests
/// and any replay of that hash stay readable. v3 entry uses
/// [`mark_decision`].
pub fn triggered(band: &Band, bid: Option<f64>, ask: Option<f64>) -> Option<super::targets::Dir> {
    use super::targets::Dir;
    let buy = ask.is_some_and(|a| a > band.upper);
    let sell = bid.is_some_and(|b| b < band.lower);
    match (buy, sell) {
        (true, false) => Some(Dir::Buy),
        (false, true) => Some(Dir::Sell),
        _ => None,
    }
}

/// How long after a 30-minute mark a quote still counts as that mark's
/// decision. The paper checks at the mark. Quotes arrive continuously; this
/// window takes the quote that arrives with the mark and refuses the rest.
pub const MARK_WINDOW_SECS: i64 = 5;

/// Whether `now` is a v3 decision mark: a multiple of 30 minutes after the
/// session open, within [`MARK_WINDOW_SECS`] of that instant, and not before
/// the first mark.
pub fn at_decision_mark(session: &Session, now: DateTime<Utc>) -> bool {
    let elapsed = now - session.open;
    let mins = elapsed.num_minutes();
    if mins < MARK_MINUTES || mins % MARK_MINUTES != 0 {
        return false;
    }
    let into_mark = elapsed.num_seconds() - mins * 60;
    (0..MARK_WINDOW_SECS).contains(&into_mark)
}

/// The exact mark belonging to an accepted quote; keep the quote's real
/// timestamp separately when recording the decision.
pub fn decision_mark(session: &Session, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    at_decision_mark(session, now).then(|| {
        session.open
            + Duration::minutes((now - session.open).num_minutes() / MARK_MINUTES * MARK_MINUTES)
    })
}

/// The v3 side at a mark: the mid buys above the upper band and sells below
/// the lower. Both sides are required. A mid inside the band, a one-sided
/// quote, or a quote wider than the band (mid can sit inside while the
/// sides straddle) enters nothing.
pub fn mark_decision(
    band: &Band,
    bid: Option<f64>,
    ask: Option<f64>,
) -> Option<super::targets::Dir> {
    use super::targets::Dir;
    let (bid, ask) = (bid?, ask?);
    if !(bid.is_finite() && ask.is_finite() && ask >= bid) || ask - bid > band.upper - band.lower {
        return None;
    }
    let mid = (bid + ask) / 2.0;
    if mid > band.upper {
        Some(Dir::Buy)
    } else if mid < band.lower {
        Some(Dir::Sell)
    } else {
        None
    }
}

/// Whether a new entry may be taken at `at`: after the first mark, before the
/// session's last 30 minutes (the session-end exit would close it at once),
/// and outside lunch.
pub fn entry_window(s: &Session, at: DateTime<Utc>) -> Result<(), String> {
    if at < s.open + Duration::minutes(MARK_MINUTES) {
        return Err("before the first 30 minutes of the session have passed".into());
    }
    if at >= s.close - Duration::minutes(MARK_MINUTES) {
        return Err("the session's last 30 minutes: no new entry (the session-end exit would close it at once)".into());
    }
    if s.lunch.is_some_and(|(a, b)| at >= a && at < b) {
        return Err("the lunch break".into());
    }
    if at >= s.close {
        return Err("the session has closed".into());
    }
    Ok(())
}
```

</details>

## §9. Exact legacy target allocation module

Source: `crates/jiaoyi-core/src/strategy3/targets.rs` at `25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f`. Excerpt SHA256: `567b820b1b3523b907cbc9c5a18906233b1080781cdeb756e72154982e789e42`.

<details>
<summary>Read archived source — historical evidence only</summary>

```rust
//! Strategy 3's exits: bot-trade's `risk_coverage_partial_v1`, ported line
//! by line from `agent/services/momentum-target-policy.js`
//! (`planMomentumTargets`, owner-approved for Strategy 3, 25-10-2026).
//!
//! With R the risk (entry to stop), C the cost reserve per unit and Q the
//! required reward multiple (`REQUIRED_RR`, 1.38 from 27-10-2026):
//! - TP1 (the trigger) = entry ± (Q·R + C), rounded outward to a whole tick;
//! - the close fraction at TP1 is f = (R + C) / (TP1 distance + R), the
//!   volume rounded up to the step and at least the broker minimum;
//! - the runner's target TP2 = TP1 ± R, sent as the broker take-profit;
//! - a position that cannot be split (the rest would fall below the
//!   minimum) closes whole at TP1, and the broker take-profit is TP1;
//! - a TP1 distance not above C is refused (`target_price_invalid`).
//!
//! The coverage identity: the part closed at TP1 pays for the runner's loss
//! at the original stop, after the reserve: close × (TP1 distance − C) ≥
//! runner × (R + C). The modelled fills are at the planned prices; gaps and
//! slippage may exceed the reserve.

use serde::Serialize;

/// bot-trade's policy name.
pub const POLICY: &str = "risk_coverage_partial_v1";
/// `MOMENTUM_MAX_PRICE_DIGITS`: a relative bracket carries at most 5 decimals.
pub const MAX_PRICE_DIGITS: i32 = 5;
/// `MOMENTUM_MIN_REQUIRED_RR`: TP1 sits Q·R + C from the entry.
///
/// **Owner, 27-10-2026: target near 1.4R.** Q = 1.38, the goal's payoff
/// reference. The previous hard minimum was 3.0. The 27 Oct statements
/// showed 3R + C (median C ≈ 2.2R, barrier ≈ 5R) cannot produce 73% wins
/// on a no-drift process. A further change of Q is a further registration.
/// See `docs/preregistration-2026-10-27-strategy3-v3.md`.
pub const REQUIRED_RR: f64 = 1.38;
const SNAP_TOLERANCE_TICKS: f64 = 1e-6;

/// `side`: buy or sell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Dir {
    Buy,
    Sell,
}

impl Dir {
    fn sign(self) -> f64 {
        match self {
            Dir::Buy => 1.0,
            Dir::Sell => -1.0,
        }
    }
}

/// The plan's inputs. Volumes are broker cents of units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Input {
    pub side: Dir,
    pub entry: f64,
    pub original_stop: f64,
    pub required_rr: f64,
    /// C: the cost reserve per unit, in price units.
    pub cost_reserve: f64,
    pub digits: i32,
    pub volume: i64,
    pub min_volume: i64,
    pub step_volume: i64,
}

/// `mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    PartialRunner,
    WholePositionMinimum,
}

/// The plan.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Plan {
    pub mode: Mode,
    pub initial_risk: f64,
    /// TP1.
    pub trigger: f64,
    /// The broker's take-profit: TP2 for a partial plan, TP1 for a whole one.
    pub broker_target: f64,
    /// TP2 (the runner's target), whatever the mode.
    pub runner_target: f64,
    pub volume: i64,
    pub close_volume: i64,
    pub runner_volume: i64,
    pub close_fraction: f64,
    pub modeled_net_at_original_stop: f64,
}

fn finite(n: f64) -> bool {
    n.is_finite()
}

fn positive(n: f64) -> bool {
    n.is_finite() && n > 0.0
}

fn price_digits(d: i32) -> bool {
    (0..=MAX_PRICE_DIGITS).contains(&d)
}

/// JavaScript's `Math.round` (half up), for the non-negative values used here.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// `priceTicks`: a price in whole ticks, or `None`.
pub fn price_ticks(price: f64, digits: i32) -> Option<i64> {
    if !finite(price) || !price_digits(digits) {
        return None;
    }
    let t = js_round(price * 10f64.powi(digits));
    (t.abs() < 9_007_199_254_740_991.0).then_some(t as i64)
}

/// `gridTicks`: whole ticks of a price within float residue of the grid.
fn grid_ticks(price: f64, digits: i32) -> Option<f64> {
    if !finite(price) || !price_digits(digits) {
        return None;
    }
    let scaled = price * 10f64.powi(digits);
    let nearest = js_round(scaled);
    ((scaled - nearest).abs() < SNAP_TOLERANCE_TICKS).then_some(nearest)
}

/// `outward`.
fn outward(price: f64, digits: i32, direction: f64) -> f64 {
    let factor = 10f64.powi(digits);
    let scaled = price * factor;
    let nearest = js_round(scaled);
    let integer = if (scaled - nearest).abs() < 1e-8 {
        nearest
    } else if direction == 1.0 {
        scaled.ceil()
    } else {
        scaled.floor()
    };
    integer / factor
}

/// `outwardTicks`.
fn outward_ticks(scaled: f64) -> f64 {
    let nearest = js_round(scaled);
    if (scaled - nearest).abs() < 1e-8 {
        nearest
    } else {
        scaled.ceil()
    }
}

/// `targets`: (trigger, runner target).
fn targets(i: &Input, risk: f64, direction: f64) -> (f64, f64) {
    let Some(entry_ticks) = grid_ticks(i.entry, i.digits) else {
        let trigger = outward(
            i.entry + direction * (i.required_rr * risk + i.cost_reserve),
            i.digits,
            direction,
        );
        return (
            trigger,
            outward(trigger + direction * risk, i.digits, direction),
        );
    };
    let factor = 10f64.powi(i.digits);
    let risk_ticks = match grid_ticks(i.original_stop, i.digits) {
        None => risk * factor,
        Some(stop_ticks) => direction * (entry_ticks - stop_ticks),
    };
    let trigger_ticks = outward_ticks(i.required_rr * risk_ticks + i.cost_reserve * factor);
    (
        (entry_ticks + direction * trigger_ticks) / factor,
        (entry_ticks + direction * (trigger_ticks + outward_ticks(risk_ticks))) / factor,
    )
}

/// `planMomentumTargets`. `Err` is bot-trade's refusal reason.
pub fn plan(i: &Input) -> Result<Plan, &'static str> {
    if !positive(i.entry)
        || !positive(i.original_stop)
        || !positive(i.required_rr)
        || !finite(i.cost_reserve)
        || i.cost_reserve < 0.0
    {
        return Err("risk_and_cost_evidence_required");
    }
    if i.required_rr < REQUIRED_RR {
        return Err("required_rr_below_hard_minimum");
    }
    if !(0..=8).contains(&i.digits) {
        return Err("price_precision_required");
    }
    if !price_digits(i.digits) {
        return Err("relative_bracket_precision_unsupported");
    }
    if i.volume <= 0
        || i.min_volume <= 0
        || i.step_volume <= 0
        || i.volume < i.min_volume
        || i.volume % i.step_volume != 0
    {
        return Err("broker_volume_invalid");
    }
    let direction = i.side.sign();
    let risk = direction * (i.entry - i.original_stop);
    if risk.is_nan() || risk <= 0.0 {
        return Err("original_stop_on_wrong_side");
    }
    let (trigger, runner_target) = targets(i, risk, direction);
    let distance = direction * (trigger - i.entry);
    if !positive(trigger) || distance.is_nan() || distance <= i.cost_reserve {
        return Err("target_price_invalid");
    }
    let fraction = (risk + i.cost_reserve) / (distance + risk);
    let step = i.step_volume as f64;
    let desired = ((i.volume as f64 * fraction).max(i.min_volume as f64) / step).ceil() as i64
        * i.step_volume;
    let partial = desired < i.volume && i.volume - desired >= i.min_volume;
    if partial && (!positive(runner_target) || direction * (runner_target - trigger) <= 0.0) {
        return Err("target_price_invalid");
    }
    let close_volume = if partial { desired } else { i.volume };
    let runner_volume = i.volume - close_volume;
    let modeled = close_volume as f64 * (distance - i.cost_reserve)
        - runner_volume as f64 * (risk + i.cost_reserve);
    if !modeled.is_finite() || modeled < -1e-7 {
        return Err("coverage_not_representable");
    }
    Ok(Plan {
        mode: if partial {
            Mode::PartialRunner
        } else {
            Mode::WholePositionMinimum
        },
        initial_risk: risk,
        trigger,
        broker_target: if partial { runner_target } else { trigger },
        runner_target,
        volume: i.volume,
        close_volume,
        runner_volume,
        close_fraction: close_volume as f64 / i.volume as f64,
        modeled_net_at_original_stop: modeled,
    })
}
```

</details>

## §9a. Exact shared relative-stop rounding helper

Source: `crates/jiaoyi/src/trading.rs` at `25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f`. Excerpt SHA256: `f959f4e0ab087f33df8c7b54f795f865f23fa96290d75f94d852740d39847209`.

<details>
<summary>Read archived source — historical evidence only</summary>

```rust
pub fn relative_stop(distance: f64, digits: i32) -> i64 {
    let tick = 10_i64.pow((5 - digits.clamp(0, 5)) as u32);
    let e5 = (distance * 100_000.0).round() as i64;
    ((e5 + tick / 2) / tick).max(1) * tick
}
```

</details>

## §10. Exact legacy cost-reserve module

Source: `crates/jiaoyi-core/src/strategy3/mod.rs` at `25bed03b36aedeb95ba4e5eb2283e98e8fd2c17f`. Excerpt SHA256: `02c01a1f08ff501a4804ba965a3a3c7ab5b74404e38dd43e13bc3fdf93f88dff`.

<details>
<summary>Read archived source — historical evidence only</summary>

```rust
//! Strategy 3, intraday momentum (owner-approved rules, 25-10-2026): the
//! pure parts.
//!
//! - [`sessions`]: each symbol's sessions in UTC, with daylight saving time
//!   from the IANA database;
//! - [`band`]: the noise band, open × (1 ± σ), at any instant;
//! - [`targets`]: the exits, bot-trade's `risk_coverage_partial_v1`.
//!
//! The cost reserve and the slippage it charges are here too.

pub mod band;
pub mod kelly;
pub mod sessions;
pub mod targets;

use crate::universe::AssetClass;

/// Per-side slippage, in basis points of price, by class: bot-trade's
/// `agent/config/momentum-target-slippage.json` (owner-confirmed 02-10-2026,
/// reply to № 10,585·B·5). bot-trade's classes map as: equity_index →
/// `index_cfd`, fx → `fx`, metal and energy → `commodity`, crypto →
/// `crypto`. Every class there is 24 bps (three of them charged the dearest
/// measured class for too few rows).
pub fn slippage_bps_per_side(class: AssetClass) -> f64 {
    match class {
        AssetClass::EquityIndex => 24.0, // index_cfd
        AssetClass::Fx => 24.0,          // fx
        AssetClass::Metal => 24.0,       // commodity
        AssetClass::Energy => 24.0,      // commodity
        AssetClass::Crypto => 24.0,      // crypto
    }
}

/// The cost reserve C per unit, in price units: the median spread, the
/// round-trip commission per unit and the slippage of both sides at `price`.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct CostReserve {
    pub spread: f64,
    pub commission_round_trip: f64,
    pub slippage: f64,
    pub total: f64,
}

pub fn cost_reserve(
    median_spread: f64,
    commission_round_trip: f64,
    slippage_bps_per_side: f64,
    price: f64,
) -> Result<CostReserve, String> {
    for (name, v) in [
        ("median spread", median_spread),
        ("round-trip commission", commission_round_trip),
        ("slippage", slippage_bps_per_side),
        ("price", price),
    ] {
        if !(v.is_finite() && v >= 0.0) {
            return Err(format!("the {name} is not known ({v})"));
        }
    }
    let slippage = 2.0 * slippage_bps_per_side / 10_000.0 * price;
    Ok(CostReserve {
        spread: median_spread,
        commission_round_trip,
        slippage,
        total: median_spread + commission_round_trip + slippage,
    })
}
```

</details>

## §11. Restoration boundary

The warning applies to future agents and humans reading this archive: do not copy the archived dispatcher into the active entry path, add a flag that silently restores it, or treat its presence as a pending implementation task. A new owner approval must identify the intended policy change and version. Historical evidence and existing-position exit protection must remain readable and attributable.
