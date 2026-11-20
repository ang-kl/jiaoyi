//! №18,392: read-only preparation while armed, bounded send on a later touch.
//! No order is sent by this worker. Durable consumption and intent stay before
//! the socket write, and the writer checks the original receipt expiry again.
use super::*;
use tokio::sync::watch;

#[derive(Clone)]
pub(super) struct Request {
    id: i64,
    name: String,
    arm: ArmedPullback,
    band: Band,
    limits: VolumeLimits,
    spread: f64,
    commission_rt: f64,
    rate: f64,
    opening_quote: Option<Value>,
    quote_at: DateTime<Utc>,
    source_at: tokio::time::Instant,
    held_risk: f64,
    s3_positions_open: usize,
}
#[derive(Clone)]
pub(super) struct Prepared {
    request: Request,
    rules: Rules,
    equity: f64,
    sized: Sized,
    order: Order,
    entry: Entry,
    command: Command,
    policy: Policy,
    admission: gate::PreparedEntry,
    kelly: Value,
    key: String,
}
#[derive(Clone, Default)]
pub(super) struct Cache {
    pub plans: HashMap<i64, Prepared>,
    pub errors: HashMap<i64, String>,
    pub attempts: HashMap<i64, Value>,
}

fn same_number(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1.0)
}

/// A restored arm's stored arithmetic does not replace the original verified
/// opening quote and causal preparation. Require both before publishing or
/// consuming a prepared admission, including after a carried-window restart.
pub(super) fn validate_source(
    name: &str,
    day: &Day,
    arm: ArmedPullback,
) -> Result<(), &'static str> {
    if day.session != arm.session {
        return Err("prepared source has wrong original window");
    }
    let open = day
        .open
        .as_ref()
        .map_err(|_| "prepared opening midpoint unavailable")?;
    if *open <= 0.0 || !same_number(*open, arm.origin) {
        return Err("prepared opening midpoint conflicts with restored arm");
    }
    let quote = day
        .opening_quote
        .as_ref()
        .ok_or("prepared opening quote evidence missing")?;
    let (Some(bid), Some(ask), Some(mid)) = (
        quote["bid"].as_f64(),
        quote["ask"].as_f64(),
        quote["mid"].as_f64(),
    ) else {
        return Err("prepared opening quote prices missing");
    };
    if bid <= 0.0
        || ask < bid
        || !same_number(mid, bid + (ask - bid) / 2.)
        || !same_number(mid, *open)
    {
        return Err("prepared opening quote prices or source invalid");
    }
    let parse = |key: &str| {
        quote[key]
            .as_str()
            .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
            .map(|at| at.with_timezone(&Utc))
    };
    let (Some(bid_at), Some(ask_at), Some(observed)) =
        (parse("bid_at"), parse("ask_at"), parse("observed_at"))
    else {
        return Err("prepared opening quote timestamps missing");
    };
    let start = prod::opening_observation_start(name, arm.session);
    let window_seconds = match quote["source"].as_str() {
        Some("recorded_ticks_native_open_6s") => 6,
        Some("recorded_ticks_native_open_15s_v18425")
            if quote["window_policy"].as_str() == Some(prod::OPENING_QUOTE_POLICY)
                && parse("observation_start") == Some(start)
                && parse("window_end")
                    == Some(
                        start + chrono::Duration::seconds(prod::OPENING_QUOTE_WINDOW_SECONDS),
                    ) =>
        {
            prod::OPENING_QUOTE_WINDOW_SECONDS
        }
        _ => return Err("prepared opening quote source or window policy invalid"),
    };
    let end = start + chrono::Duration::seconds(window_seconds);
    if [bid_at, ask_at]
        .iter()
        .any(|at| *at < start || *at > end || *at > arm.armed_at)
        || observed != bid_at.max(ask_at)
    {
        return Err("prepared opening quote is stale or outside original window");
    }
    let context = *day
        .macro_context
        .as_ref()
        .map_err(|_| "prepared causal macro context unavailable")?;
    if !mp::aligns(context, arm.long, arm.session.open)
        || context.latest_bar_end != arm.macro_context.latest_bar_end
        || !same_number(context.drift_5, arm.macro_context.drift_5)
        || !same_number(context.drift_20, arm.macro_context.drift_20)
    {
        return Err("prepared macro context conflicts with restored arm or is noncausal");
    }
    Ok(())
}

pub(super) fn source_refusal(sym: &Sym) -> Option<String> {
    let arm = sym.pullback?;
    match sym.pullback_day(arm) {
        Some(day) => validate_source(&sym.name, day, arm)
            .err()
            .map(str::to_owned),
        _ => Some("prepared original-window evidence unavailable".into()),
    }
}

impl Trader<'_, '_> {
    pub(super) fn warm_requests(&self, now: DateTime<Utc>) -> Vec<Request> {
        let Ok(book) = self.commission.as_ref() else {
            return vec![];
        };
        if self
            .commission_read
            .is_none_or(|at| at.elapsed() >= COMMISSION_REFRESH)
        {
            return vec![];
        }
        let Ok(held_risk) = self.open_risk() else {
            return vec![];
        };
        let s3_positions_open = self.own.len() + self.pending.len();
        let mut requests = vec![];
        for sym in self.syms.values() {
            if self.entry_policy.s3_refusal(self.env, &sym.name).is_some()
                || self.human_entry_refusal(sym.id).is_some()
                || self.pair_entry_refusal(sym.id).is_some()
                || sym
                    .rule
                    .and_then(|rule| sessions::session_at(&rule, now))
                    .is_none_or(|window| {
                        prod::symbol_entry_window(&sym.name, &window, now).is_err()
                    })
            {
                continue;
            }
            let Some(arm) = sym.pullback else { continue };
            let Some(day) = sym.pullback_day(arm) else {
                continue;
            };
            if validate_source(&sym.name, day, arm).is_err() {
                continue;
            }
            let (Some(limits), Some(rate), Some((bid, ask))) =
                (sym.limits, self.rate(sym.id), fresh_s3_quote(sym, now))
            else {
                continue;
            };
            let Ok(band) = jiaoyi_core::strategy3::production_sessions::band_at(
                &day.table,
                &arm.session,
                arm.origin,
                arm.armed_at,
            ) else {
                continue;
            };
            let per_unit = book
                .per_class
                .get(sym.class.as_str())
                .map_or(book.overall_median, |v| v.0);
            let Some(quote_at) = sym.bid_at.zip(sym.ask_at).map(|(b, a)| b.min(a)) else {
                continue;
            };
            let Ok(quote_age) = (now - quote_at).to_std() else {
                continue;
            };
            let Some(source_at) = tokio::time::Instant::now().checked_sub(quote_age) else {
                continue;
            };
            requests.push(Request {
                id: sym.id,
                name: sym.name.clone(),
                arm,
                band,
                limits,
                spread: ask - bid,
                commission_rt: 2.0 * per_unit / rate,
                rate,
                opening_quote: day.opening_quote.clone(),
                quote_at,
                source_at,
                held_risk,
                s3_positions_open,
            });
        }
        requests.sort_by_key(|r| r.id);
        requests
    }

    pub(super) fn fast_plan(&self, id: i64, arm: ArmedPullback) -> Option<Prepared> {
        let sym = self.syms.get(&id)?;
        let day = sym.pullback_day(arm)?;
        validate_source(&sym.name, day, arm).ok()?;
        let plan = self.fast_cache.plans.get(&id)?;
        if plan.request.arm.armed_at != arm.armed_at
            || plan.request.arm.session != arm.session
            || !same_number(plan.request.arm.extreme, arm.extreme)
            || !same_number(plan.request.arm.impulse_range, arm.impulse_range)
            || !same_number(plan.request.arm.level, arm.level)
            || plan
                .admission
                .validate(
                    self.session,
                    &PgLedger {
                        pool: self.pool,
                        settings_version: Some(plan.rules.version),
                    }
                    .admission_namespace(),
                    &plan.command,
                    &plan.policy,
                )
                .is_err()
        {
            return None;
        }
        Some(plan.clone())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) async fn dispatch_prepared(
        &mut self,
        id: i64,
        arm: ArmedPullback,
        bid: f64,
        ask: f64,
        now: DateTime<Utc>,
        deadline: tokio::time::Instant,
        plan: Prepared,
        outcome: &mut TriggerOutcome,
    ) -> anyhow::Result<()> {
        if let Some(reason) = self
            .human_entry_refusal(id)
            .or_else(|| self.entry_policy.s3_refusal(self.env, &self.syms[&id].name))
        {
            outcome.refuse(reason);
            return Ok(());
        }
        let started = tokio::time::Instant::now();
        let name = plan.request.name.clone();
        let sym = self.syms[&id].clone();
        let _admission = crate::admission::lock(self.account_id()).await;
        if self.pair_entry_refusal(id).is_some() {
            outcome.refuse("pair_risk_reservation_active");
            self.skip(
                id,
                &name,
                "pair_risk_reservation_active",
                json!({"armed":arm}),
                now,
            )
            .await;
            return Ok(());
        }
        if sym
            .rule
            .and_then(|rule| sessions::session_at(&rule, now))
            .is_none_or(|window| prod::symbol_entry_window(&name, &window, now).is_err())
        {
            outcome.refuse("current_execution_window_unavailable");
            return Ok(());
        }
        let expires = deadline.min(plan.admission.expires());
        if plan.rules.live {
            let slots = tokio::time::timeout_at(
                expires,
                crate::admission::live_s3_concurrency(
                    self.pool,
                    self.session,
                    LIVE_MAX_S3_POSITIONS,
                ),
            )
            .await
            .unwrap_or_else(|_| Err("live: Strategy 3 slot check exceeded dispatch expiry".into()));
            if let Err(reason) = slots {
                outcome.refuse(&reason);
                self.skip(id, &name, &reason, json!({"armed":arm}), now)
                    .await;
                return Ok(());
            }
        }
        // The equity was read for this receipt and has the same five-second
        // expiry. Recheck the currently held risk under the dispatch lock.
        let held = self.open_risk().map_err(anyhow::Error::msg)?;
        let budget = within_budget(
            held,
            plan.sized.risk,
            plan.equity,
            effective_open_risk_pct(&plan.rules),
        );
        if let Err(reason) = budget {
            outcome.refuse(&reason);
            self.skip(id, &name, &reason, json!({"armed":arm}), now)
                .await;
            return Ok(());
        }
        let (_, peer) = self.peer_evidence(&name, now);
        let Some(proof) = self.writer_turn_proof(arm, bid, ask, now) else {
            outcome.refuse("confirmed_pullback_proof_unavailable");
            return Ok(());
        };
        let guard = jiaoyi_ctrader::entry_guard::EntryGuard {
            connection_generation: plan.admission.generation(),
            account_revision: plan.admission.revision(),
            account_id: self.account_id(),
            symbol_id: id,
            side: dir_side(plan.entry.side),
            signal: jiaoyi_ctrader::entry_guard::Signal::PullbackConfirmed {
                origin: arm.origin,
                level: arm.level,
                impulse_range: arm.impulse_range,
                admitted_entry: plan.entry.entry_price,
                price_digits: plan.request.limits.digits,
                peer,
                proof,
                margin: plan.admission.receipt.fx_guard.clone(),
                calendar: Some(plan.admission.calendar.clone()),
            },
            cost: Some(jiaoyi_ctrader::entry_guard::Cost {
                prepared_spread: plan.request.spread,
                commission_round_trip: plan.request.commission_rt,
                slippage_bps_per_side: mp::LIVE_SLIPPAGE_BPS_PER_SIDE,
                original_r: arm.impulse_range,
            }),
            expires,
        };
        let exit_deadline = sym
            .rule
            .as_ref()
            .and_then(|rule| sessions::session_at(rule, now))
            .map(|window| prod::exit_deadline(&sym.name, &window))
            .ok_or_else(|| anyhow::anyhow!("no active execution window"))?;
        let detail = json!({"production_policy":mp::POLICY,"model":mp::MODEL,"armed":arm,
            "session_date":arm.session.date,"session":arm.session,"decided_at":now,"bid":bid,"ask":ask,"mid":(bid+ask)/2.,
            "exit_deadline":exit_deadline,"exit_deadline_policy":prod::EXIT_DEADLINE_POLICY,
            "side":plan.entry.side,"band":plan.request.band,"open":arm.origin,"macro_context":arm.macro_context,
            "volume_limits":plan.request.limits,"cost":plan.entry.cost,"entry":plan.entry,
            "opening_quote":plan.request.opening_quote,"quote_to_deposit":plan.request.rate,
            "sizing":plan.sized,"order":plan.order,"kelly":plan.kelly,"at_ms":now.timestamp_millis(),
            "trigger_quote":{"bid":bid,"ask":ask,"bid_at":sym.bid_at,"ask_at":sym.ask_at},"prepared_admission":{"target_pre_dispatch_ms":250,"planned_entry":plan.entry.entry_price,"settings_version":plan.rules.version,"margin":plan.admission.receipt},
            "dispatch_policy":jiaoyi_ctrader::entry_guard::POLICY});
        let ledger = S3Ledger {
            inner: PgLedger {
                pool: self.pool,
                settings_version: Some(plan.rules.version),
            },
            order: s3s::OrderRow {
                account_id: self.account_id(),
                entry_key: plan.key.clone(),
                label: LABEL.into(),
                prereg_hash: self.hash.clone(),
                kind: "open".into(),
                symbol: name.clone(),
                symbol_id: id,
                position_id: None,
                side: Some(dir_side(plan.entry.side).as_str().into()),
                session_date: Some(arm.session.date),
                decided_at: now,
                bid: Some(bid),
                ask: Some(ask),
                volume: Some(plan.sized.volume),
                detail: detail.clone(),
            },
            recorded_at: Mutex::new(None),
        };
        self.d.version = Some(plan.rules.version);
        self.syms.get_mut(&id).unwrap().last_try = Some(now);
        outcome.sending(&plan.key);
        let result = gate::send_prepared_entry(
            self.session,
            self.allowlist,
            &ledger,
            &plan.policy,
            plan.command.clone(),
            &plan.key,
            Duration::from_secs(plan.rules.order_wait_secs),
            &guard,
            &plan.admission,
        )
        .await;
        self.syms.get_mut(&id).unwrap().admission_restored_window = None;
        // The intent receipt records when the socket path began; the whole
        // await also includes the broker response and is not dispatch latency.
        let intent_at = *ledger.recorded_at.lock().unwrap();
        let latency = intent_at.map(|at| (at - now).num_milliseconds());
        match result {
            Ok(sent) => {
                outcome.sent(&sent);
                self.syms.get_mut(&id).unwrap().entered_session = Some(arm.session.open);
                self.record_fill(
                    &ledger,
                    &plan.key,
                    "open",
                    &name,
                    None,
                    dir_side(plan.entry.side),
                    Some(plan.entry.entry_price),
                    Some(bid),
                    Some(ask),
                    plan.sized.volume,
                    &sent,
                )
                .await?;
                if matches!(&sent.outcome,Outcome::Rejected{code:Some(code),..} if code=="NOT_ENOUGH_MONEY")
                {
                    self.funding_retry_at =
                        Some(now + chrono::Duration::from_std(ENTRY_RETRY).unwrap_or_default());
                }
                if matches!(sent.outcome, Outcome::Filled { .. }) || partly_filled(&sent) {
                    self.reconcile(Utc::now()).await?;
                } else if matches!(sent.outcome, Outcome::Unconfirmed) {
                    self.pending.push(Pending {
                        entry_key: plan.key.clone(),
                        symbol_id: id,
                        risk: plan.sized.risk,
                        at: now,
                    });
                }
                self.d.note(Some(&name),"sent",&format!("{STRATEGY} prepared {} {} cents: {:?}",dir_side(plan.entry.side).as_str(),plan.sized.volume,sent.outcome),json!({"entry_key":plan.key,"touch_to_intent_ms":latency,"response_wait_ms":started.elapsed().as_millis(),"relative_stop":plan.entry.relative_stop})).await;
            }
            Err(SendError::Refused(e)) => {
                outcome.refuse(&e.reason);
                if e.reason.contains("ENTRY_COST_GE_R") {
                    self.syms.get_mut(&id).unwrap().cost_skip_session = Some(arm.session.open);
                }
                self.d.note(Some(&name), "refused", &e.reason, detail).await
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}

async fn prepare(
    pool: &PgPool,
    allowlist: &Allowlist,
    env: Environment,
    session: &AccountSession<'_>,
    hash: &str,
    q: Request,
) -> Result<Prepared, String> {
    let age = Utc::now() - q.quote_at;
    if age < chrono::Duration::zero() || age > chrono::Duration::seconds(5) {
        return Err("armed quote for preparation stale or future".into());
    }
    let id = session.account().id();
    let (r, opened, equity, keys, attempts) = tokio::join!(
        rules(pool, id, env),
        s3s::opens_in_window(pool, id, q.id, q.arm.session.open, hash),
        fresh_equity(session),
        s3s::open_keys_for_hash(pool, id, q.id, hash),
        s3s::attempts_in_window(pool, id, q.id, q.arm.session.open, hash)
    );
    let r = r.map_err(|e| e.to_string())?;
    if r.paused {
        return Err("owner pause is active".into());
    }
    if session_already_entered(opened.map_err(|e| e.to_string())?) {
        return Err("session already entered".into());
    }
    let price =
        mp::dispatch_entry_bound(&q.arm, q.spread, q.limits.digits).map_err(str::to_owned)?;
    let cost = cost_reserve(
        q.spread,
        q.commission_rt,
        mp::LIVE_SLIPPAGE_BPS_PER_SIDE,
        price.max(q.arm.origin),
    )?;
    if cost.total >= q.arm.impulse_range {
        return Err("prepared cost >= impulse risk".into());
    }
    let geometry = mp::dispatch_geometry(&q.arm, q.spread, cost.total, q.limits.digits)
        .map_err(str::to_owned)?;
    let rel = relative_stop(geometry.geometry.risk, q.limits.digits);
    let equity = equity?;
    let keys = keys.map_err(|e| e.to_string())?;
    let sample = kelly_sample(pool, id, &keys)
        .await
        .map_err(|e| e.to_string())?;
    let kelly = jiaoyi_core::strategy3::kelly::read(sample.as_deref().unwrap_or_default());
    if jiaoyi_core::strategy3::kelly::skip(&kelly) {
        return Err("Kelly fraction not positive".into());
    }
    let sized = size(&r, equity, rel, q.rate, &q.limits, q.s3_positions_open)?;
    within_budget(q.held_risk, sized.risk, equity, effective_open_risk_pct(&r))?;
    let entry = Entry {
        side: if q.arm.long { Dir::Buy } else { Dir::Sell },
        band: q.band,
        entry_price: geometry.admitted_entry,
        relative_stop: rel,
        r: rel as f64 / 100000.,
        cost,
    };
    let order = pullback_order(&entry, &q.limits, sized.volume)?;
    let command = Command::Open {
        symbol_id: q.id,
        side: dir_side(entry.side),
        volume: sized.volume,
        relative_stop: rel,
        relative_take_profit: Some(order.relative_take_profit),
        limits: q.limits,
        quote_to_deposit: Some(q.rate),
        label: LABEL.into(),
    };
    let policy = Policy {
        edge: Ok(()),
        max_risk: sized.max_risk,
    };
    let n = attempts.map_err(|e| e.to_string())?;
    let key = format!(
        "s3:{id}:{}:{}:{}",
        q.id,
        q.arm.session.open.timestamp(),
        n + 1
    );
    let ledger = PgLedger {
        pool,
        settings_version: Some(r.version),
    };
    // S3 uses its own slot count here and a fresh check under the send lock.
    // The last argument selects the legacy account-wide one-trade rule only.
    let mut admission = gate::prepare_entry(session, allowlist, &ledger, &policy, &command, false)
        .await
        .map_err(|e| e.to_string())?;
    admission.restrict_source_start(q.source_at);
    admission
        .validate(session, &ledger.admission_namespace(), &command, &policy)
        .map_err(|e| e.to_string())?;
    Ok(Prepared {
        request: q,
        rules: r,
        equity,
        sized,
        order,
        entry,
        command,
        policy,
        admission,
        kelly: json!({"n":kelly.n,"p":kelly.p,"f_star":kelly.f_star,"history_queried":sample.is_some(),"writes_size":false}),
        key,
    })
}

pub(super) async fn run(
    pool: &PgPool,
    allowlist: &Allowlist,
    env: Environment,
    session: &AccountSession<'_>,
    mut requests: watch::Receiver<Vec<Request>>,
    output: watch::Sender<Cache>,
    mut shutdown: Shutdown,
) -> anyhow::Result<()> {
    let hash = prereg_hash();
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut cache = Cache::default();
    let mut cursor = 0usize;
    loop {
        tokio::select! {_=shutdown.wait()=>return Ok(()),_=tick.tick()=>{}}
        let qs = requests.borrow_and_update().clone();
        cache.plans.retain(|id, p| {
            qs.iter()
                .any(|q| q.id == *id && q.arm.armed_at == p.request.arm.armed_at)
                && p.admission.expires() > tokio::time::Instant::now()
        });
        if qs.is_empty() {
            cache.errors.clear();
            output.send_replace(cache.clone());
            continue;
        }
        // Bound concurrent read-only work to two armed symbols. No account
        // admission lock is held while waiting, and the core quote/exit actor
        // continues independently. Rotate larger sets without starving peers.
        let mut batch = Vec::with_capacity(2);
        for _ in 0..qs.len().min(2) {
            cursor %= qs.len();
            batch.push(qs[cursor].clone());
            cursor += 1;
        }
        let mut jobs = batch.into_iter();
        let run = |q: Option<Request>| async {
            match q {
                Some(q) => {
                    let id = q.id;
                    let started = tokio::time::Instant::now();
                    let source_age_ms = started.duration_since(q.source_at).as_millis();
                    let result = prepare(pool, allowlist, env, session, &hash, q).await;
                    let detail = json!({"completed_at":Utc::now(),
                        "source_age_at_start_ms":source_age_ms,
                        "preparation_duration_ms":started.elapsed().as_millis(),
                        "prepared_successfully":result.is_ok(),
                        "failure":result.as_ref().err(),
                        "evidence_scope":"last completed read-only preparation; final writer guards still apply"});
                    Some((id, result, detail))
                }
                None => None,
            }
        };
        let publish = |row: Option<(i64, Result<Prepared, String>, Value)>| {
            let Some((id, result, detail)) = row else {
                return;
            };
            cache.attempts.insert(id, detail);
            match result {
                Ok(p) => {
                    cache.plans.insert(id, p);
                    cache.errors.remove(&id);
                }
                Err(e) => {
                    cache.plans.remove(&id);
                    cache.errors.insert(id, e);
                }
            }
            output.send_replace(cache.clone());
        };
        tokio::select! {
            _=shutdown.wait()=>return Ok(()),
            _=publish_independently(Box::pin(run(jobs.next())), Box::pin(run(jobs.next())), publish)=>{}
        }
    }
}

// A slower sibling must not hold a fresh five-second admission until expiry.
// There are still at most two read-only jobs; neither receives order authority.
async fn publish_independently<T>(
    first: impl std::future::Future<Output = T>,
    second: impl std::future::Future<Output = T>,
    mut publish: impl FnMut(T),
) {
    tokio::pin!(first, second);
    tokio::select! {
        value = &mut first => { publish(value); publish(second.await); }
        value = &mut second => { publish(value); publish(first.await); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiaoyi_core::secret::Secret;
    use jiaoyi_ctrader::messages::{
        ProtoOaInterval, ProtoOaLightSymbol, ProtoOaPosition, ProtoOaSymbol, ProtoOaTradeData,
        ProtoOaTrader,
    };
    use jiaoyi_ctrader::testkit::FakeBroker;

    #[tokio::test]
    async fn faster_admission_is_published_before_slow_sibling_finishes() {
        let (release, delayed) = tokio::sync::oneshot::channel();
        let (output, mut received) = tokio::sync::mpsc::unbounded_channel();
        let work =
            publish_independently(async { "ready" }, async { delayed.await.unwrap() }, |v| {
                output.send(v).unwrap();
            });
        let observe = async {
            assert_eq!(
                tokio::time::timeout(Duration::from_secs(2), received.recv())
                    .await
                    .unwrap(),
                Some("ready")
            );
            assert!(received.try_recv().is_err());
            release.send("slow_result").unwrap();
            assert_eq!(received.recv().await, Some("slow_result"));
        };
        tokio::join!(work, observe);
    }

    fn armed_source(session: Session) -> (ArmedPullback, Day) {
        let open = session.open;
        let context = mp::Context {
            drift_5: 0.0123456789012345,
            drift_20: 0.0234567890123456,
            latest_bar_end: open - chrono::Duration::hours(20),
        };
        let arm = ArmedPullback {
            long: true,
            origin: 100.,
            extreme: 101.,
            level: 100.5,
            impulse_bps: 100.,
            impulse_range: 1.,
            armed_at: open + chrono::Duration::minutes(5),
            session,
            macro_context: context,
            confirmation: Default::default(),
        };
        let quote = json!({"source":"recorded_ticks_native_open_6s","bid":99.99,"ask":100.01,"mid":100.,"bid_at":open,"ask_at":open,"observed_at":open});
        let day = Day {
            session,
            open: Ok(100.),
            opening_quote: Some(quote),
            table: SigmaTable {
                marks: [(30, Ok(0.01))].into_iter().collect(),
                sessions_used: vec![],
            },
            spread: Ok((0.02, 1)),
            prepared_at: open,
            macro_context: Ok(context),
        };
        (arm, day)
    }

    #[sqlx::test(migrator = "jiaoyi_store::MIGRATOR")]
    async fn live_preparation_uses_s3_slots_and_dispatch_checks_the_fresh_book(pool: PgPool) {
        let account = 7;
        let id = 1;
        let allowlist = Allowlist::parse(Some("7")).unwrap();
        jiaoyi_store::upsert_account(&pool, account, Environment::Live)
            .await
            .unwrap();
        crate::trading::apply_seed(&pool, crate::trading::SEED_FILE)
            .await
            .unwrap();
        let position = |symbol_id, label: &str, volume| ProtoOaPosition {
            position_id: symbol_id + 10,
            position_status: 1,
            trade_data: ProtoOaTradeData {
                symbol_id,
                trade_side: 1,
                volume,
                label: Some(label.into()),
                ..Default::default()
            },
            price: Some(100.0),
            stop_loss: Some(99.25),
            used_margin: Some(volume as u64),
            money_digits: Some(2),
            ..Default::default()
        };
        let fake = FakeBroker {
            account_id: account,
            trader: Some(ProtoOaTrader {
                ctid_trader_account_id: account,
                balance: 111_006,
                deposit_asset_id: 1,
                money_digits: Some(2),
                account_type: Some(0),
                ..Default::default()
            }),
            symbols: vec![ProtoOaSymbol {
                symbol_id: id,
                schedule_time_zone: Some("UTC".into()),
                trading_mode: Some(0),
                schedule: vec![ProtoOaInterval {
                    start_second: 0,
                    end_second: 7 * 86_400,
                }],
                ..Default::default()
            }],
            light_symbols: (1..=4)
                .map(|symbol_id| ProtoOaLightSymbol {
                    symbol_id,
                    quote_asset_id: Some(1),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let positions = fake.positions.clone();
        positions
            .lock()
            .unwrap()
            .extend([position(2, "jiaoyi-s1", 100), position(3, "", 100)]);
        let (conn, log) = fake.spawn();
        let session = conn
            .account_auth(&allowlist.permit(account).unwrap(), &Secret::new("token"))
            .await
            .unwrap();
        let mut trader = Trader::with_entry_policy(
            &pool,
            &allowlist,
            Environment::Live,
            &session,
            crate::config::EntryPolicy {
                crypto_4h: false,
                live_execution: true,
            },
        )
        .await
        .unwrap();
        let date = "2026-11-09".parse().unwrap();
        let (arm, day) = armed_source(prod::windows_on(prod::Market::Index, date)[0]);
        let now = arm.armed_at;
        trader.reconcile(now).await.unwrap();
        assert_eq!(trader.program_positions, 1);
        trader.deposit = Some("USD".into());
        trader.commission = Ok(costs::CommissionBook {
            per_class: Default::default(),
            overall_median: 0.0,
            deals: 1,
        });
        trader.commission_read = Some(std::time::Instant::now());
        trader.syms.insert(
            id,
            Sym {
                id,
                name: "US500".into(),
                class: AssetClass::EquityIndex,
                rule: prod::rule_for("US500", AssetClass::EquityIndex),
                limits: Some(VolumeLimits {
                    min: 1_000,
                    step: 1_000,
                    max: None,
                    digits: 2,
                }),
                quote: Some("USD".into()),
                bid: Some(100.49),
                ask: Some(100.51),
                bid_at: Some(now),
                ask_at: Some(now),
                day: Some(Ok(day)),
                preopen: None,
                last_try: None,
                last_decision_mark: None,
                entered_session: None,
                cost_skip_session: None,
                pullback: Some(arm),
                pullback_source_day: None,
                pullback_loaded_session: None,
                last_assessment: None,
                admission_restored_window: None,
                last_impulse_check: None,
            },
        );
        let request = |trader: &Trader<'_, '_>| {
            let mut request = trader.warm_requests(now).pop().expect("armed US500");
            // The source window uses a fixed event clock. Its broker receipt
            // uses the real clock, as in a wire replay.
            request.quote_at = Utc::now();
            request
        };
        assert_eq!(request(&trader).s3_positions_open, 0);

        let held = position(4, LABEL, 5_000);
        positions.lock().unwrap().push(held.clone());
        // Hold the reservation while the cached intent awaits reconciliation
        // with its now-visible broker fill.
        trader.pending.push(Pending {
            entry_key: "s3:7:4:1".into(),
            symbol_id: 4,
            risk: 37.50,
            at: now,
        });
        let next = request(&trader);
        assert_eq!(next.s3_positions_open, 1);
        assert_eq!(next.held_risk, 37.50);
        let plan = prepare(
            &pool,
            &allowlist,
            Environment::Live,
            &session,
            &trader.hash,
            next,
        )
        .await
        .expect("S1 and an unrelated S3 position permit a prepared plan");
        assert_eq!(plan.sized.mode, "live_broker_minimum");
        assert_eq!(plan.admission.receipt.floor_pct, Some(250.0));

        // A larger reservation added after warming must veto the same plan.
        trader.pending[0].risk = 55.0;
        let mut outcome = TriggerOutcome::pending();
        trader
            .dispatch_prepared(
                id,
                arm,
                100.49,
                100.51,
                now,
                tokio::time::Instant::now() + Duration::from_secs(5),
                plan.clone(),
                &mut outcome,
            )
            .await
            .unwrap();
        let outcome = json!(outcome);
        assert_eq!(outcome["execution_resolution"], "not_sent");
        assert!(outcome["reason"]
            .as_str()
            .unwrap()
            .contains("would exceed its budget 55.50 (5% of equity)"));

        // The slot check must see exposure that appeared since preparation.
        let mut another = held;
        another.position_id += 1;
        positions.lock().unwrap().push(another);
        let mut outcome = TriggerOutcome::pending();
        trader
            .dispatch_prepared(
                id,
                arm,
                100.49,
                100.51,
                now,
                tokio::time::Instant::now() + Duration::from_secs(5),
                plan,
                &mut outcome,
            )
            .await
            .unwrap();
        let outcome = json!(outcome);
        assert_eq!(outcome["execution_resolution"], "not_sent");
        assert!(outcome["reason"]
            .as_str()
            .unwrap()
            .contains("2 Strategy 3 position(s)/unresolved open(s) are active"));
        assert!(log.lock().unwrap().orders.is_empty());
    }

    #[test]
    fn restored_source_requires_original_quote_and_causal_matching_macro() {
        let open = DateTime::parse_from_rfc3339("2026-11-09T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let (arm, day) = armed_source(Session {
            date: open.date_naive(),
            open,
            close: open + chrono::Duration::hours(4),
            lunch: None,
        });
        let mut restored: ArmedPullback =
            serde_json::from_str(&serde_json::to_string(&arm).unwrap()).unwrap();
        restored.macro_context.drift_5 += 1e-16;
        assert!(
            validate_source("BTCUSD", &day, restored).is_ok(),
            "JSON rounding must preserve a valid restored receipt"
        );
        for milliseconds in [6000, 7100, 15000, 15001] {
            let mut extended = day.clone();
            let quote = extended.opening_quote.as_mut().unwrap();
            let at = open + chrono::Duration::milliseconds(milliseconds);
            quote["bid_at"] = json!(at);
            quote["ask_at"] = json!(at);
            quote["observed_at"] = json!(at);
            assert_eq!(
                validate_source("BTCUSD", &extended, arm).is_ok(),
                milliseconds <= 6000,
                "legacy policy must never acquire the extended window implicitly"
            );
            let quote = extended.opening_quote.as_mut().unwrap();
            quote["source"] = json!("recorded_ticks_native_open_15s_v18425");
            quote["window_policy"] = json!(prod::OPENING_QUOTE_POLICY);
            quote["observation_start"] = json!(open);
            quote["window_end"] = json!(open + chrono::Duration::seconds(15));
            assert_eq!(
                validate_source("BTCUSD", &extended, arm).is_ok(),
                milliseconds <= 15000
            );
            extended.opening_quote.as_mut().unwrap()["window_policy"] = json!("unknown");
            assert!(validate_source("BTCUSD", &extended, arm).is_err());
        }
        let mut bad = day.clone();
        bad.open = Err("missing".into());
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
        let mut bad = day.clone();
        bad.opening_quote = None;
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
        let mut bad = day.clone();
        bad.session.open += chrono::Duration::hours(4);
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
        let mut bad = day.clone();
        bad.open = Ok(101.);
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
        let mut bad = day.clone();
        bad.opening_quote.as_mut().unwrap()["bid_at"] = json!(open - chrono::Duration::seconds(1));
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
        let mut bad = day.clone();
        bad.macro_context = Err("missing".into());
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
        let mut bad = day.clone();
        bad.macro_context.as_mut().unwrap().latest_bar_end = open + chrono::Duration::seconds(1);
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
        let mut bad = day.clone();
        bad.macro_context.as_mut().unwrap().drift_20 = -0.01;
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
        let mut bad = day;
        bad.macro_context.as_mut().unwrap().drift_5 = 0.04;
        assert!(validate_source("BTCUSD", &bad, arm).is_err());
    }
}
