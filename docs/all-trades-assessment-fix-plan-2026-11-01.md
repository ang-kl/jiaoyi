# Fix plan: all-trades assessment and final-entry validation

**Status:** Draft plan for review; no implementation, build, broker mutation or deployment performed by this document.

**Prepared:** 1 November 2026, 10:25 SGT (Asia/Singapore); reviewed at 10:29 SGT. **Trace:** №18,090–18,092; incorporates the verified findings in №18,087 §§1–6 and owner clarification in №18,089 §§1–5. **By:** <Codex (GPT-6), effort: not exposed>.

**Inspected baseline:** `95e631adf316fe6c90a0986d88c9673175f82809`, the PR146 merge. Deployment identity was verified during the earlier read-only investigation; this planning turn does not refresh market/account state.

**Reference gap:** The owner explicitly requests inclusion of **№18,083**. Its exact message was not recovered from the available chat or saved Markdown/JSON evidence. The owner has been asked once for its text. This plan incorporates the preserved subsequent investigation, but does **not** claim those findings are a verbatim or complete reconstruction of №18,083. Add its exact requirements to the traceability table when supplied; do not invent them or declare that reference closed.

## §PLAN.1 — Intended result

Jiaoyi assesses every observed position and pending entry on every configured account, including human-created trades and manual changes, on a **one-second schedule and after material broker events**. It states which checks pass, fail or remain unknown, why, and how current the evidence is. Ownership determines authority to act, not coverage of assessment.

A second correction checks that an entry remains valid at the last controlled point before transmission. A third workstream tests proposed direction, quantity and exit changes against preserved trade paths before any new trading policy is activated.

The financial invariants remain **risk:reward 1:1.38 and at least 73% net wins over the latest 20 qualified fully closed owned positions, separately for each strategy/account**. At least 15 wins are required in a 20-position window. Original planned geometry, realised net payoff and win rate are reported separately. Monitoring latency and passed CI are engineering metrics, not financial progress.

## §PLAN.2 — Requirements and evidence

| Reference / finding | Included response | Completion evidence |
|---|---|---|
| №18,083, exact wording unavailable | Reserved requirement row; reconcile the supplied text with this plan | Exact source plus mapped task/check; currently open |
| №18,087 §2: preparation and protection were confused with pending entries | Separate preparation, pending entry, open position and protective closing order | T01–T02 |
| №18,087 §§3–4: stops moved, but direction reversal and continuous TP retuning do not exist | Describe actual policies; assess current suitability; evaluate alternatives separately | T10–T11 and economic gates E1–E4 |
| №18,087 §5: entry quote can age during awaited work | Final pre-write validation, including ledger and writer-lock delays | T11–T13 |
| №18,089: assess manual and program trades every second | Account-wide independent observer and evaluator | T01–T10, T14–T15 |
| Existing attribution repairs | Preserve original ownership/risk lineage; keep current exposure separate | T03, T07–T08, T10 |
| Existing queue lessons, GitHub issue147 | Reconcile duplicate work and named waits; do not silently dispatch jobs | Work coordination in §PLAN.9 |

The earlier investigation found successful SL tightening in all four selected latest S3 closes. It did not demonstrate that a reversal or different TP would have won. No attributable new S3 close after the 31 October repair deployment was found in that bounded read. Those are dated observations, not claims about the account state when implementation begins.

## §PLAN.3 — Phase A: establish complete, current assessment inputs

**Owner:** account-state implementer; independently reviewed for event races and attribution.

1. Build one read-only account snapshot from the full broker book, including protection orders, and account-level execution events. Reuse existing connections; ensure collection continues when a particular strategy is disabled. Inspect the existing report collector and avoid a competing polling loop. `AccountSession::margin_book()` already requests protection orders.
2. Classify every object as open position, pending entry or protective closing order. Keep origin/ownership, observed side/quantity/SL/TP and applicable strategy separate. Labels alone do not confer ownership. Preparation rows are not broker orders.
3. Subscribe to prices and metadata for every held/pending symbol, including instruments outside the strategy universe and any required currency conversions. Coordinate subscription ownership so observer cleanup cannot unsubscribe a strategy's feed. Subscription or metadata failure creates an explicit unknown result.
4. Use connection generation and increasing local book revisions. A broker snapshot arriving late must not overwrite a newer manual change. Buffer/reconcile events received during snapshot acquisition; bound that buffer. Partial events must not erase fields they did not supply. Duplicate events must not duplicate exposure.
5. On a disconnect, dropped event or ambiguous change, invalidate affected evidence and request one reconciliation at a time. Keep unknown exposure visible while recovery is pending. Do not resurrect closed positions from old snapshots or silently discard unmatched pending orders.
6. Preserve original opening evidence, original R and registration separately from current exposure. A human quantity change or reversal can make strategy attribution unknown even though the current position still requires assessment.

**Output:** immutable, revisioned snapshots plus quote/metadata caches. No new broker writer, trading authority or per-second broker polling.

## §PLAN.4 — Phase B: independent one-second assessment

**Owner:** assessment implementer; must consume Phase A snapshots rather than duplicate the account book.

Run a dedicated deterministic Rust evaluator inside the existing `jiaoyi` service. Its critical path performs no database, broker, history or model awaits. It reads a coherent snapshot, evaluates it, then publishes a result carrying the exact book/plan and quote revisions and ages. A material position/order change invalidates a superseded result. A newer quote alone does not discard every completed result: coalesce it into the next evaluation so continuous ticks cannot starve publication. Coalesce event bursts; do not accumulate one job per tick.

```text
Broker book + execution events + quotes + metadata
                         |
                         v
              Revisioned account snapshot
                         |
              every second / material change
                         v
               Pure setup/risk assessment
                         |
                         v
       Latest verdict + reasons + evidence timestamps
                         |
          compact changes / owner report / review

Registered trading actors -> existing authorised order gate -> cTrader
```

The evaluator assesses; the existing actors retain their execution responsibilities. The observer has no amend, close, cancel or reverse capability.

### Assessment fields

| Dimension | Required interpretation |
|---|---|
| Market/entry direction | Supported / contradicted / unknown against an explicitly identified strategy and reference time. For S3, distinguish the last registered mark decision from an indicative current comparison. A one-second comparison does not create a one-second entry strategy. |
| Manual trade without a strategy | Risk and protection remain assessable; strategy suitability is unknown. Do not silently assign S3 or label the trade bad because S3 would not enter it. |
| Quantity and account exposure | Current broker units, lot conversion, step/minimum, applicable risk budget and whole-account margin/concentration. Apply only policies that actually govern that object. The live program-position rule is not permission to relabel human positions. |
| Stop protection | Presence, executable-side geometry, accepted current broker value, nominal loss at stop and any plan violation. A profitable trailing stop is valid on the profitable side of entry. Stop-based estimates do not cap actual gap/slippage losses. |
| Target and reward/risk | Original-plan consistency and cost-adjusted target where measurable. S1's absence of a fixed TP is not automatically a defect. An external trade without TP has unknown target reward, not an invented target. |
| Pending entry | Conditional risk at its trigger, current side support, SL/TP geometry, broker quantity validity and capacity if triggered. A pending trigger being far from market is not by itself a failed setup. |
| Eligibility | Separate valid market conditions from permission/capacity to enter: session, ownership, owner pause, cost veto, existing exposure and evidence quality. |
| Response | Named reason and applicable registered response, or a draft recommendation for review. Do not present an untested reversal/target as an execution instruction. |

Keep individual known breaches visible even when another component is unknown. “Passes configured checks” must never be displayed as “will be profitable”. Do not manufacture probabilities or expected returns from the goal.

### Cadence and evidence freshness

- Schedule evaluation every 1,000 ms using monotonic time; show completed-at time, event-to-evaluation delay, missed deadlines and p95/p99/max completion intervals. A heartbeat is not an assessment completion.
- Reuse the existing six-second quote-age limit initially as a named observation policy, subject to verification against instrument sessions. A one-second tick does not refresh an old quote. Track bid and ask receipt times independently; stale, crossed, missing or future-dated inputs cannot certify executable conditions.
- Show last complete broker reconciliation time, last relevant event time, connection generation and book revision separately. Reuse the existing reconciliation cadence; invalidate immediately on known stream gaps. Define and test the maximum book-age policy from the actual configured cadence before implementation closure, rather than implying one-second broker truth.
- Closed market, unavailable metadata and unsupported instrument remain explicit states. A known protection breach remains visible even when the market is closed.
- In the controlled load test, complete each scheduled assessment within its one-second interval despite a deliberately held 30-second strategy/history future. Production missed deadlines remain visible and trigger a bounded incident; they are never hidden by averaging.

## §PLAN.5 — Phase C: final-entry validation

**Priority:** highest execution correction. Build in parallel with Phases A/B where interfaces allow, and release independently if its evidence is ready first.

The existing strategy captures a quote before awaited preparation/account checks. Additional waits occur when persisting the intent and acquiring the transport writer lock. The final guard must therefore run **after lock acquisition, immediately before starting the write**, using locally available evidence. No slow work belongs under that lock. Once transmission has begun, the system cannot guarantee the market remains unchanged.

1. Carry the original decision mark/time, registration, frozen band/levels, selected side, entry key and explicit validation policy with the proposed open. Retain the original strategy context; do not substitute a newly calculated band.
2. Before recording the exact intent, validate fresh price/quantity/risk and account state, with cost calculated from the registered formula and permitted inputs. If repricing requires recalculation, complete that calculation through the existing gates before persistence. Never send a payload that differs from its recorded intent.
3. At the final pre-write point, recheck quote freshness, original-side validity, validation expiry and relevant snapshot revisions. A material change requiring slow recalculation vetoes this attempt; it does not cause computation while holding the writer lock.
4. Use S3's actual `mark_decision` midpoint semantics with the original frozen band. Do not substitute the historical `triggered` helper. Changed conditions can refuse the entry; they cannot flip the side, refresh the consumed mark or add a second entry.
5. Preserve current S1 signal semantics and exit rules. The new entry guard must not accidentally block an already-authorised close or stop amendment.
6. If an intent is persisted but the guard refuses before any transmission, record a durable `refused_before_send` outcome. Recovery, risk reservations and session-entry counting must recognise this proven-unsent outcome without reusing the same intent. If recording that outcome fails, retain uncertainty and resolve it; do not assume the reservation is free.
7. A partial write, timeout after starting transmission or late broker acceptance remains unresolved until authoritative evidence resolves it. Never convert it into an unsent refusal or automatically submit a duplicate.

**Cost semantics:** current S3 cost reserve uses prepared spread evidence. Substituting instantaneous spread into that reserve would change its policy and must be explicitly versioned. The observer may display current spread separately. If the registered formula establishes `C ≥ R`, preserve the durable session cost veto. Quote-age, dispatch-expiry and changed-direction refusals have separate reason codes and must not create that cost veto accidentally. The final check cannot silently redefine which costs qualify as a session-wide rejection.

**Proposed dispatch policy, requiring explicit versioning:** S3's current five-second window defines eligible decision quotes; it is not already a dispatch TTL. A maximum send time of mark + five seconds would be a stricter execution condition. Benchmark its impact under normal and delayed account/ledger calls, document it in a new registration/version, and present missed-entry rates before activation. S1 also needs a documented dispatch-age policy suited to its decision semantics. This plan does not silently activate either policy or change Q, size, stops or the durable session cost veto.

## §PLAN.6 — Phase D: reporting and evidence from actual trades

Publish compact latest assessments through the existing authorised owner view/report. Public health exposes coverage counts, freshness and completion timing without private positions or task payloads. Show coverage by account: observed positions/entries, assessed objects, failed checks, unknown checks and oldest evidence. A denominator must include external and unsupported symbols.

Persist only material verdict changes and bounded evidence needed to reproduce them: object/revision, registration or manual-unassigned status, reason, source times and relevant changed values. Retain existing fills/history rather than copying them into a new telemetry archive. Set and test retention/size bounds during implementation. Unchanged one-second evaluations update memory, not a database row or Telegram message.

For a close, preserve the latest setup assessment, original plan, manual changes, actual protection acknowledgements, fills and costs. Classify the observed issue before assigning engineering work: accounting/ownership defect, stale or incomplete data, execution failure, policy restriction, or unsupported economic hypothesis. A loss alone is not a bug report or automatic model-session trigger.

Learning runs on changed material evidence under the existing allowance; repeated quotes and unchanged incidents do not consume sessions. Reuse issue147 to reconcile queue duplicates and completed PR144–146 work. Inspect full incident capacity before relying on a new incident delivery path; a source-level capacity concern is not proof a particular live incident was dropped.

## §PLAN.7 — Phase E: validate proposed calibration separately

Continuous observation does not supply a validated rule for reversing, resizing or moving TP. Use the existing research plan, with four explicit economic gates:

| Gate | Work | Acceptance |
|---|---|---|
| E1: reconstruct | Join original intent/version, manual edits, price path, protection changes, partial exits, fills and costs | Sufficient timestamped evidence for the tested claim; gaps labelled, no invented price path |
| E2: specify | Define a small candidate set: original policy versus a precise thesis-invalidation exit/reversal, volatility-aware sizing or target rule | Freeze candidate, baseline, costs and comparison method before scoring; no arbitrary tweaking of every loser |
| E3: evaluate | Replay appropriate historical controls, then use unseen/shadow observations; account for copied episodes and spread/slippage | Independent reproduction; report losses, tail risk, after-cost outcomes and missing coverage. Tick-path claims require adequate tick evidence |
| E4: decide | Keep, reject or propose a versioned policy based on evidence | A demonstrated improvement is required; any new authority/risk values explicitly handled. No automatic live activation from this document |

The S3 session-wide cost veto and one-entry rule remain visible reasons for inactivity. Their opportunity cost can be evaluated in E2–E3; they are not removed simply to increase trade count. Broker-minimum live size cannot be reduced below the broker minimum. A proposed adjustment must report when no permissible smaller quantity exists.

Report official latest-20 goal windows separately per strategy/account, excluding unqualified external/mixed lifecycles. Also report frozen-version evidence separately: a historical combined window does not validate a new policy. Engineering repairs cannot close E1–E4 or prove a persistent edge.

## §PLAN.8 — Fifteen engineering acceptance scenarios

These are **planned checks, not passed tests**. Pair each failure case with a legitimate-case control where applicable.

| ID | Scenario | Required result |
|---|---|---|
| T01 | Manual position and pending entry arrive | Both assessed on every account; no management authority acquired |
| T02 | Protective closing order and preparation row | Neither counted as a pending new entry or duplicate exposure |
| T03 | Human changes side, quantity, SL/TP; partial close/reversal | Old assessment invalidated; new revision uses actual state; original-risk attribution not fabricated |
| T04 | External symbol/conversion outside strategy universe | Shared subscription/metadata obtained or explicit unknown; strategy subscription retained |
| T05 | Stale/missing/crossed/one-sided quotes, closed market | No fabricated pass; known independent breaches remain visible |
| T06 | Snapshot returns after newer manual event | Newer state wins; superseded assessment cannot overwrite it |
| T07 | Duplicate/partial/out-of-order/dropped events | No double count; partial-field handling correct; gaps trigger one bounded reconciliation |
| T08 | Reconnect and process restart | Generation changes invalidate old evidence; full bootstrap; no inferred authority |
| T09 | Slow history/DB/broker work and event bursts | Independent one-second evaluation continues without starvation; measured deadlines/lag exposed |
| T10 | Manual-unassigned strategy, profitable trailing SL, missing TP, unit conversion | Correct per-check verdicts and nominal risk arithmetic for both sides; no false S1 TP breach or units/lots confusion |
| T11 | Direction/cost changes during entry awaits | Original-side invalidity refuses entry; correct midpoint and registered-cost semantics; cost veto distinct from age/expiry refusal; no flip or same-mark retry; valid control still sends |
| T12 | Ledger or writer-lock delay beyond proposed expiry | Proven-unsent refusal is durable; zero order frame; reservations/recovery correct; next permissible decision not permanently blocked; exits unaffected |
| T13 | Partial write, timeout, late broker acceptance | Unresolved handling and intent deduplication preserved; no false unsent status |
| T14 | Continuous ticks, high volume and repeated unchanged verdicts | Snapshot-consistent results continue publishing; bounded memory/storage, single-flight reads, no per-second AI/Telegram calls; observer emits zero trading frames |
| T15 | Exact deployed version, all-account coverage and observed timing | Independent release readback, denominators complete, source ages honest; no unintended settings/order changes |

## §PLAN.9 — Work order, roles and release gates

| Stage | Main-agent checkpoint | Delegation / output |
|---|---|---|
| 0. Baseline | Refresh main, other-model log and runtime identities; reconcile №18,083 if supplied | Main agent maps confirmed requirements; evidence agent reuses existing receipts |
| 1. Inputs + final entry | Approve data/guard interfaces and reproduce failures | Account-state agent handles A; execution agent handles C; independent reviewer challenges races |
| 2. Assessment | Verify full inventory before judging coverage | Assessment agent implements B using A; no second account book |
| 3. Evidence/report | Verify owner can distinguish bad setup, blocked entry and unknown evidence | Reporting/accounting agent handles D; reuse existing private route |
| 4. Release | CI green, exact deployed source and T15 verified | Main agent owns merge/deployment closure; independent reviewer checks receipts |
| 5. Economics | Resume existing research at first unmet gate | Research/accounting agents handle E; no requirement to use every daily permit |

Use at most two implementation PRs if separable: **final-entry validation** and **all-trades assessment/reporting**. Further PRs require a demonstrated independent change. Do not create a PR solely for a speculative alternative strategy. Every PR carries chat №/§, SGT time and by<model,effort>; update `log/CODEX_Log.txt` and check `log/GROK_Log.txt` first.

Targeted behavioural tests come first, followed by the repository's required formatting, Clippy, Rust/PostgreSQL, coordinator and image checks. Do not weaken CI to conceal a trading-state failure. Merge only green under the owner's standing merge instruction when implementation is authorised; a plan is not a claim that implementation or financial validation passed.

Release the observer in read-only mode with its verdicts clearly labelled. Verify all configured accounts and natural manual/broker events; do not create live test orders merely to populate a demonstration. Use fake-broker/demo fixtures for destructive/race scenarios. Complete a bounded 30-minute healthy-stream observation, then a relevant active-market session report; missing events remain unverified rather than extending a model session indefinitely.

For entry validation, verify the exact versioned dispatch policy and its normal-case latency before enabling it. Use one authoritative writer during deployment. Rollback/disable only the new assessment or guard component through a documented release path; preserve existing broker protection and open-position management. A rollback is not permission to pause accounts, cancel manual orders or weaken unrelated risk limits.

The Railway engineering worker's mutation window and permissions remain separate from this chat. Do not silently renew them or dispatch the backlog. Each assigned job records scope, input revision, authority, next checkpoint and completion condition. An idle wait must identify the expected event and resume condition; a missed checkpoint becomes a named incident.

## §PLAN.10 — Cost, boundaries and completion

Use the existing Rust service, broker streams, database/reporting routes and model allowance. No new service/framework or paid model route is needed for the per-second evaluator. Extra symbol subscriptions and cached computations have infrastructure cost; measure their count/latency and bound them. Do not call an LLM every second or consume all permits merely because available.

Engineering completion for the known scope requires A–D implemented, T01–T15 passed with exact deployed receipts, observable coverage/latency and preserved execution boundaries. E1–E4 remain separately tracked with honest results. Known work can complete independently of the missing reference; **“all requested scope completed” remains open until №18,083 is recovered and mapped**. Report implemented, verified, awaiting evidence and economically unproven separately. Never declare “no more bugs” or that fifteen checks prove profitability.

**Time is Money.** “It's not the will to win that matters—everyone has that. It's the will to prepare to win that matters.” Preserve serial/paragraph numbering, SGT date/time, truthful evidence and the financial goals throughout.

## Source map

- [Current system contract](current-system-contract.md), [S3 v6 registration](preregistration-2026-10-28-strategy3-v6.md), [research plan](research-validation-plan-2026-10-31.md).
- [Fast monitoring implementation/proposal](fast-position-monitor-proposal.md); historical proposal text does not override later registered trailing rules.
- `crates/jiaoyi/src/fast_monitor.rs`: three-second progress observer and quote-age handling.
- `crates/jiaoyi/src/trading.rs` and `strategy3.rs`: owned-position management, subscriptions, mark decision, sizing and entry dispatch.
- `crates/jiaoyi-core/src/strategy3/band.rs`: actual mark/midpoint decision and five-second quote window.
- `crates/jiaoyi-ctrader/src/trading.rs` and `client.rs`: shared gate, persisted intent and transport writer sequence.
- [Consolidated queue issue147](https://github.com/ang-kl/jiaoyi/issues/147): job triage and existing evidence plan; not a worker dispatch.
- Private local evidence: `/workspace/jiaoyi-pretrade-review-20261101/accounting/findings.md`, `evidence.json`, `all-trades-assessment-requirement.md`, and `/workspace/article-project-swap-20261101/queued-lessons-and-job-triage.md`. Financial payloads stay outside public repository reports.

**Draft for review.**
