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
    program_positions: usize,
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
        let mut requests = vec![];
        for sym in self.syms.values() {
            let Some(arm) = sym.pullback else { continue };
            let Some(Ok(day)) = sym.day.as_ref() else {
                continue;
            };
            if day.session != arm.session {
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
                program_positions: self.program_positions + self.pending.len(),
            });
        }
        requests.sort_by_key(|r| r.id);
        requests
    }

    pub(super) fn fast_plan(&self, id: i64, arm: ArmedPullback) -> Option<Prepared> {
        let plan = self.fast_cache.plans.get(&id)?;
        if plan.request.arm.armed_at != arm.armed_at
            || plan.request.arm.session != arm.session
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
    ) -> anyhow::Result<()> {
        let started = tokio::time::Instant::now();
        let name = plan.request.name.clone();
        let sym = self.syms[&id].clone();
        let _admission = if plan.rules.live {
            Some(crate::admission::lock(self.account_id()).await)
        } else {
            None
        };
        let held = self.open_risk().map_err(anyhow::Error::msg)?;
        if let Err(reason) =
            within_budget(held, plan.sized.risk, plan.equity, plan.rules.open_risk_pct)
        {
            self.skip(id, &name, &reason, json!({"armed":arm}), now)
                .await;
            return Ok(());
        }
        let (_, peer) = self.peer_evidence(&name, now);
        let trigger_quote = jiaoyi_ctrader::entry_guard::Quote {
            bid: sym.bid_at.map(|at| (bid, at)),
            ask: sym.ask_at.map(|at| (ask, at)),
        };
        let guard = jiaoyi_ctrader::entry_guard::EntryGuard {
            connection_generation: plan.admission.generation(),
            account_revision: plan.admission.revision(),
            account_id: self.account_id(),
            symbol_id: id,
            side: dir_side(plan.entry.side),
            signal: jiaoyi_ctrader::entry_guard::Signal::PullbackBounded {
                origin: arm.origin,
                level: arm.level,
                impulse_range: arm.impulse_range,
                admitted_entry: plan.entry.entry_price,
                price_digits: plan.request.limits.digits,
                peer,
                trigger_quote: trigger_quote.clone(),
                margin: plan.admission.receipt.fx_guard.clone(),
                calendar: plan.admission.calendar.clone(),
            },
            cost: Some(jiaoyi_ctrader::entry_guard::Cost {
                prepared_spread: plan.request.spread,
                commission_round_trip: plan.request.commission_rt,
                slippage_bps_per_side: mp::LIVE_SLIPPAGE_BPS_PER_SIDE,
                original_r: arm.impulse_range,
            }),
            expires: deadline.min(plan.admission.expires()),
        };
        let exit_deadline = sym
            .rule
            .as_ref()
            .and_then(|rule| sessions::session_at(rule, now))
            .map(|window| window.close)
            .ok_or_else(|| anyhow::anyhow!("no active execution window"))?;
        let detail = json!({"production_policy":mp::POLICY,"model":mp::MODEL,"armed":arm,
            "session_date":arm.session.date,"session":arm.session,"decided_at":now,"bid":bid,"ask":ask,"mid":(bid+ask)/2.,
            "exit_deadline":exit_deadline,
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
    let sized = size(
        &r,
        equity,
        rel,
        q.rate,
        &q.limits,
        if r.live { 0 } else { q.program_positions },
    )?;
    within_budget(q.held_risk, sized.risk, equity, r.open_risk_pct)?;
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
    let mut admission = gate::prepare_entry(session, allowlist, &ledger, &policy, &command, r.live)
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
                    Some((id, prepare(pool, allowlist, env, session, &hash, q).await))
                }
                None => None,
            }
        };
        let results = tokio::select! {_=shutdown.wait()=>return Ok(()),r=async{tokio::join!(run(jobs.next()),run(jobs.next()))}=>r};
        for (id, result) in [results.0, results.1].into_iter().flatten() {
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
        }
        output.send_replace(cache.clone());
    }
}
