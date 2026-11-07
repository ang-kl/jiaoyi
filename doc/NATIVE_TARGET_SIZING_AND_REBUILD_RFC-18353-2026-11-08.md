# Native cost admission, target geometry and rebuild RFC — №18,353

№18,353 §§1–8 · Report assembled 08-11-2026 06:41:55 SGT (+08:00) · by <Codex (GPT-6), effort: not exposed> · Repository: `ang-kl/jiaoyi`

## §1. TL;DR and decision

**The requested offline evaluation is complete. None of the four cost profiles establishes a trading edge.** At the lowest assumed cost, the expanded sample has **17/52 net wins (32.69%), −0.144123R mean and PF 0.748929** with 18 unresolved trades. Forcing an executable close at the observation boundary produces **27/70 net wins (38.57%), −0.071398R mean and PF 0.844992**. All 159 cached episodes were processed, using **zero new broker requests**.

**The premise about inflated geometry was incorrect.** The two-stage model's original risk distance, initial stop, +1R entry-price-stop trigger and gross +1.38R target are independent of `PlanBasis.cost` at a fixed entry. Lower costs affect native admission and the separate production cost-inclusive target planner. The original eight entries retain identical two-stage geometry and paths: **zero losses or unresolved trades convert to wins or net breakeven**.

The cheaper profiles admit **62 additional high-R episodes** while retaining the independent 48 bp floor. Those added trades have negative mean gross returns before assumed costs. This points to an unproven selection/entry mechanism under these tested exit rules, rather than a demonstrated edge hidden solely by a high reserve. It does not establish that every possible entry or exit design must fail.

**Goal progress:** this reused development cache adds **zero independent observations** toward the owner's per-strategy/per-account latest-20 goal of at least 73% net wins and reference 1:1.38 R:R. The older comparable research ledger remains **8/20 closed trades, 3 wins and 5 unresolved entries**. The 70 entries below belong to changed admission profiles; they cannot be pooled with that ledger or labelled account qualification. §§5–8 propose **eight rebuild phases and twelve gates**, starting with contract and cost evidence.

**Delivered:** frozen plan, native offline adapter, full row-level results with provenance, comparison tables, Markdown/HTML report and rebuild RFC. The rebuild is a proposal. No live rule, risk limit, broker order or production cost setting was changed by this task. The sealed validation interval was not opened.

## §2. Frozen experiment contract and interpretation

### 2.1 What was actually evaluated

The scope is the same **119 completed seasonal episodes plus 40 event-week episodes** used in PR #189, across SpotCrude, SpotBrent, US500 and XAUUSD. Failed US500 24 January 2024 remains outside the 159; four complete US500 tick files inside the 159 lack M30 preparation and produce no entry. These are research replays of broker quotes, not newly executed broker trades. No invented ticks, return constants or substituted winners are used.

Native costs are recomputed before the decision and dispatch checks. The original durable `C >= R` session veto remains first. An independent `R / executable_price × 10,000 >= 48` floor is checked at decision and dispatch; rejection by that floor alone does not add a new whole-session veto. One entry per session remains in force. The three lower-cost arms all admit the same 70 entries in this corpus; their minimum admitted R is **48.080079 bp**.

The public research request contract still requires its existing 24 bp reserve. The experimental alternatives are restricted to this local-file adapter and the four registered core symbols. Existing default research paths retain their prior policy.

### 2.2 Geometry correction and a concrete example

`R` is the rounded historical noise-band half-width, not `C`. The retained experimental exit sets initial stop at −R, applies an entry-price stop after +R favourable excursion, and targets outward-rounded +1.38R. The existing production planner instead uses a cost-inclusive TP1 distance based on `1.38R + C`, partial risk coverage where lot constraints permit it, and a separate runner target. Applying that production policy here would change the named two-stage model and invalidate exact PR #189 baseline parity.

Therefore this task recomputes **native C and admission**, records native production target diagnostics, and retains the explicitly requested two-stage exit. A cost-scaled R/BE/gross-target experiment cannot simultaneously preserve that definition and reproduce the baseline. No claim is made that production's partial/runner target policy was evaluated as a new arm.

Example: **SpotCrude, 26 January 2023**, R = 0.72 price units, R/entry = 88.725677 bp. All four arms enter at the same time, side, price and volume:

| Cost / side | Initial stop distance | BE trigger distance | Two-stage gross TP distance | Native C | Production TP1 distance (diagnostic only) |
| --- | --- | --- | --- | --- | --- |
| 24 bp baseline | 0.720000 | 0.720000 | 0.994000 | 0.419491 | 1.414000 |
| 3.74 bp native | 0.720000 | 0.720000 | 0.994000 | 0.090696 | 1.085000 |
| 2 bp native | 0.720000 | 0.720000 | 0.994000 | 0.062458 | 1.057000 |
| Mapped: oils 3.74 / index and gold 2 | 0.720000 | 0.720000 | 0.994000 | 0.090696 | 1.085000 |

The final column is **diagnostic only**. Its reduction does not move this experiment's 0.994 gross target. An entry-price stop remains a net loss after execution costs.

### 2.3 Time boundaries and cost accounting

The directive's “4h” names inherit the PR #189 observation-boundary convention. **This is not a uniform four-hour holding-period study:** the oil native session can admit trades from 07:00 through 17:00 UTC using cached preparation/quotes; the nominal evaluation interval is 13:00–17:00. Gold/NY windows are four hours, with the existing NY DST conversion. Trades admitted early in the oil session can be observed for longer than four hours. Cropping those entries now would violate the required exact baseline comparison. This limitation is explicit in the frozen plan and applies equally to all arms.

“Strict” below means stop/TP paths within that inherited observation window, with remaining positions censored. “Forced” separately permits a time close on the first fresh executable quote at or after the boundary, within five seconds; active bracket exits retain priority. It is a counterfactual time-close result, not proof that the strict unresolved trade would naturally close there. In this sample all admitted positions obtain a forced result. No M30 continuation or 72-hour holding claim is made.

Admission uses the same causally prepared spread and cost formula. Executed quote-side prices include the recorded bid/ask spread once; net scenario R subtracts `(entry_price + exit_price) × bp_per_side / 10,000 / original_R`. The inherited extra-commission input is zero and unverified. **2 and 3.74 bp are sensitivities, not empirically calibrated broker costs.** Overnight financing and historical/live cost equivalence are not established by this intraday cached experiment. PF is computed from gains/losses in original-R units, not as an account-dollar PF.

## §3. Measured side-by-side results

### 3.1 All 159 episodes: strict observation-boundary results

| Native cost / side | Entries | Closes | Net wins | Win rate | Unresolved | Mean net R | PF in R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 24 bp baseline | 8 | 7 | 3/7 | 42.86% | 1 | -0.265185 | 0.517485 |
| 3.74 bp native | 70 | 52 | 17/52 | 32.69% | 18 | -0.202374 | 0.670115 |
| 2 bp native | 70 | 52 | 17/52 | 32.69% | 18 | -0.144123 | 0.748929 |
| Mapped: oils 3.74 / index and gold 2 | 70 | 52 | 17/52 | 32.69% | 18 | -0.198106 | 0.675464 |

Closed-only win rates omit unresolved entries from the denominator, which is why the unresolved column is essential. No-entry episodes are retained in the 159-episode population. There are no net-flat resolved outcomes.

### 3.2 All 159 episodes: forced observation-boundary close

| Native cost / side | Entries | Closes | Net wins | Win rate | Unresolved | Mean net R | PF in R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 24 bp baseline | 8 | 8 | 3/8 | 37.50% | 0 | -0.321609 | 0.436232 |
| 3.74 bp native | 70 | 70 | 26/70 | 37.14% | 0 | -0.126635 | 0.744082 |
| 2 bp native | 70 | 70 | 27/70 | 38.57% | 0 | -0.071398 | 0.844992 |
| Mapped: oils 3.74 / index and gold 2 | 70 | 70 | 27/70 | 38.57% | 0 | -0.122086 | 0.751895 |

The lowest-cost arm has the least negative net result, but **all means are negative and all PFs are below 1**. Across the three lower-cost arms the gross paths are identical: strict mean gross R is **−0.077167**, and forced mean gross R is **−0.007908**. Lower accounting charges do not turn these expanded gross results into an edge.

### 3.3 Original eight admitted episodes only

| Native cost / side | Entries | Closes | Net wins | Win rate | Unresolved | Mean net R | PF in R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 24 bp baseline | 8 | 7 | 3/7 | 42.86% | 1 | -0.265185 | 0.517485 |
| 3.74 bp native | 8 | 7 | 3/7 | 42.86% | 1 | +0.337290 | 2.631451 |
| 2 bp native | 8 | 7 | 3/7 | 42.86% | 1 | +0.389032 | 3.194238 |
| Mapped: oils 3.74 / index and gold 2 | 8 | 7 | 3/7 | 42.86% | 1 | +0.344355 | 2.724558 |

All original eight entries are unchanged. Strict transitions are four loss→loss, three win→win and one unresolved→unresolved. Forced transitions are five loss→loss and three win→win. Original-cohort forced mean net R is **−0.321609 / +0.247874 / +0.296783 / +0.257690** in the same arm order. Lower-cost native results on those eight exactly equal the same-fill repricing control. Thus the earlier positive +0.389032R sensitivity was conditional on retaining the old selected cohort; it did not predict the outcome of applying cheaper admission to all episodes.

### 3.4 Additional 62 entries within the other 151 episodes

| Native cost / side | Entries | Closes | Net wins | Win rate | Unresolved | Mean net R | PF in R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 24 bp baseline | 0 | 0 | 0/0 | Not available | 0 | Not available | Not available |
| 3.74 bp native | 62 | 45 | 14/45 | 31.11% | 17 | -0.286322 | 0.576908 |
| 2 bp native | 62 | 45 | 14/45 | 31.11% | 17 | -0.227058 | 0.642848 |
| Mapped: oils 3.74 / index and gold 2 | 62 | 45 | 14/45 | 31.11% | 17 | -0.282489 | 0.581078 |

For each lower-cost arm, **62** of these 151 episodes enter, **45** resolve and **17** remain open. Of the 45 strict closes, **14 win and 31 lose**. Mean gross R is **−0.158938**. Forced mean gross R across all 62 is **−0.054476**; net forced means are **−0.174959 / −0.118906 / −0.171089** for 3.74 / 2 / mapped costs respectively. Forced winners are 23 / 24 / 24.

Every added entry follows a prior 24 bp session cost veto. Delay from that first veto to the lower-cost entry ranges from **30.01 to 540.04 minutes**, median **180.02 minutes**. This is direct evidence that changing a durable veto alters which later opportunities survive. It is not merely a smaller accounting deduction on existing trades. No original entry was replaced or removed in this sample.

### 3.5 Symbol and cohort breakdown: mapped cost arm

| Symbol | Episodes | Entries | Strict wins/closes | Unresolved | Strict mean net R | Forced wins/closes | Forced mean net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| SpotCrude | 40 | 33 | 7/25 | 8 | -0.271715 | 11/33 | -0.177031 |
| SpotBrent | 40 | 31 | 9/23 | 8 | -0.070801 | 14/31 | -0.017314 |
| US500 | 39 | 6 | 1/4 | 2 | -0.470055 | 2/6 | -0.361211 |
| XAUUSD | 40 | 0 | 0/0 | 0 | Not available | 0/0 | Not available |

XAUUSD has **no admitted setup**; its profitability is not estimated. Oil entries dominate the expanded sample. US500's four strict closes are too sparse for a reliable class inference. Related oil prices and repeated sessions also make trade observations dependent.

| Cohort | Episodes | Entries | Strict wins/closes | Unresolved | Strict mean net R | Forced mean net R |
| --- | --- | --- | --- | --- | --- | --- |
| 2023-Q1-WINTER | 20 | 8 | 1/5 | 3 | -0.444075 | -0.223888 |
| 2023-Q3-SUMMER | 20 | 5 | 1/3 | 2 | -0.353478 | -0.076513 |
| 2024-Q1-WINTER | 19 | 6 | 6/6 | 0 | +1.262451 | +1.262451 |
| 2024-Q3-SUMMER | 20 | 10 | 2/9 | 1 | -0.281490 | -0.298947 |
| 2024_Q4_Election_Vol | 20 | 10 | 1/8 | 2 | -0.715208 | -0.460362 |
| 2025-Q1-WINTER | 20 | 9 | 3/5 | 4 | +0.286401 | +0.124065 |
| 2025-Q3-SUMMER | 20 | 7 | 0/2 | 5 | -1.132055 | -0.230477 |
| 2025_Q2_Tariff_Vol | 20 | 15 | 3/14 | 1 | -0.393446 | -0.390480 |

All-arm, per-symbol and per-cohort figures are available in the results JSON. A favourable isolated cohort, if any, is descriptive and cannot be selected retrospectively as a validated rule.

## §4. Verification, evidence quality and reproducibility

### 4.1 Admission and quality counts

| Cost / side | Cost-veto episodes | Below-48-bp episodes | Inside band / spread veto | Missing fresh decision quote | Missing next fresh fill quote |
| --- | --- | --- | --- | --- | --- |
| 24 bp baseline | 124 | 0 | 80 | 16 | 0 |
| 3.74 bp native | 11 | 113 | 130 | 41 | 5 |
| 2 bp native | 0 | 124 | 137 | 41 | 5 |
| Mapped: oils 3.74 / index and gold 2 | 0 | 124 | 137 | 41 | 5 |

Reason counts are episode counts and **overlap**; an episode may pass through several mark outcomes. They must not be added as mutually exclusive failures. Baseline statuses are 8 admitted, 147 no entry and 4 missing preparation. Lower-cost statuses are 70 admitted, 85 no entry and 4 missing preparation. The 24 bp arm suppresses sessions earlier, so it need not reach the new independent-floor check.

The result retains **20 historical protocol-flagged sources**. Three original admitted trades carry such flags, including two of the three baseline winners; each lower-cost arm has 16 protocol-flagged admitted episodes. Across the full source corpus, same-millisecond bid/ask groups occur in **159 episodes**, long update intervals in **156**, stale-side events in **89**, initial missing-side observations in **26**, and crossed quotes in **0**. These are diagnostics, not 159 proven transport failures. They constrain sequencing/freshness certainty and remain visible rather than being erased by a successful replay.

### 4.2 Checks performed

- Exact PR #189 parity for **all 159 episode admission-mark sequences, strict paths and forced-close paths** in the 24 bp arm.
- **1,143 independent cost-arithmetic checks** against emitted native outcomes; strict censoring remains separate from counterfactual closes.
- Source file hashes verified before and after the evaluation; frozen code, plan and compiled adapter hashes recorded. No new broker calls and no sealed validation opening.
- Two focused Rust checks cover native-cost admission with the independent floor/public contract and production-target versus original-R independence. The S3 tick regression group contains nine passing tests in total, including the first of these checks.
- One focused Python test checks cost-profile accounting, unresolved observations, entry identity and denominators. Core all-target Clippy with warnings denied passed; formatting and native adapter compilation passed. Full repository CI is required before merge and is recorded on the PR.

These checks establish reproducibility and selected engineering properties, not market profitability or universal absence of bugs. The adapter's explicit unqualified-source status is retained even where scenario arithmetic is measurable.

### 4.3 Provenance and reproduction boundary

Frozen code commit: `043c1c61353f8035b913e95de6a00b8a924c452a`. Plan frozen at **08-11-2026 06:29:48 SGT (+08:00)** before native replay; completed at **08-11-2026 06:30:45 SGT (+08:00)**.

Plan SHA256: `01ac6d99acfab79e7e2ea631c13863ad044d2a24ab8e10957594cc22486d284a`.

Compiled adapter SHA256: `66f083cda3e14ea7d1b79109d866366855320d76e32f5da073386b79e5130faa`.

Canonical measured rows, comparisons, summaries and same-fill controls SHA256: `24d612dd4f4d4aabae92252b7e9dd27e8eea6090bca2cd42e5f58d57f7a95c01`. The post-run assessment and report do not alter those outcomes.

Read the [frozen plan](../docs/native_target_sizing_offline_plan_18353.json), [measured results](../docs/native_target_sizing_offline_results_18353.json), [native adapter](../crates/jiaoyi-core/src/research/native_target_sizing.rs) and [offline runner](../ops/research/native_target_sizing.py). The plan references exact local source paths and hashes; raw broker cache files are retained in this workspace rather than duplicated in Git. A clean clone can inspect all published per-episode results, but must receive the matching cached files to independently rerun them. The wrapper refuses plan/source drift and will not download replacements. Re-execution is bound to the frozen source commit, not arbitrary later main code.

The report uses the user-requested serial №18,353 and unique journal entries **§LOG.143 onward** because §LOG.131 was already occupied. The existing Grok log was checked before assigning the journal entries. Engineering completion and financial qualification remain separate decisions.

## §5. Unvarnished critique of the hypotheses and research process

### 5.1 The acute problem is an unproven predictive mechanism, compounded by uncertain economics

The work has demonstrated reproducible calculations and several useful engineering repairs. It has not demonstrated a deployable trading edge. Correct implementation of a band breakout, a fade, a trailing stop or a target ratio is not evidence that any of them predicts a profitable conditional return distribution.

There have been materially different experiments, not a single steadily improving strategy:

| Evidence stage | What it established | What it did not establish |
| --- | --- | --- |
| №18,270: fixed-horizon diagnostics | Some oil, metal and index directions worked on a small November sample; the retained review reproduced the observations. | A directional checkpoint win is not a full SL/TP/trailing trade, an independent validation sample or demonstrated value from a peer filter. |
| №18,278–18,281: preparation and full-exit research | Provenance and missing-input reporting improved; full-path replay separated feasibility, signal and exit failures. | Better scorecards did not improve account performance. Selected oil observations were too small to establish an edge. |
| №18,307–18,325: native preparation and seasonal cohorts | Native oil preparation became executable; multiple years and explicit controls widened the evidence. | A downloaded tick file did not establish complete preparation, a representative sample, cost calibration or equal candidate/control exposure. |
| №18,331: dynamic admission and two-stage exits | The two-stage exit changed outcomes on five old entries. Dynamic admission admitted many more trades. | Five closes, including flagged sources, did not validate the exit. The dynamic two-stage arm returned 6/83 wins and negative mean gross R; merely increasing activity did not solve prediction. |
| №18,338–18,348: peers, cost audit and high-volatility weeks | Missing peers and preparation could be distinguished from an absent signal; fixed high-volatility cohorts produced measured negative outcomes. | Naming an event week “high volatility” did not ensure that the model's decision-time historical band exceeded its cost hurdle. The lower cost values were not empirically calibrated. |
| №18,351: fade and hold | Reversing identical entries worsened this sample; cost sensitivities changed net arithmetic. | The 72-hour extension had no continuation bars for its one in-scope open trade. Its equality to baseline did not show that longer holding works or fails. |
| №18,353: this experiment | Costs are recomputed inside native admission, with an independent 48 bp floor and exact baseline parity. | Reusing the same development observations cannot qualify the account goal or validate a cost assumption selected because it improves a result. |

Sources: [№18,270 review](../docs/ai-review-2026-11-06/reviews/Response_18270.md), [Phase A](../docs/phase-a-implementation-2026-11-06.md), [№18,281 assessment](AI-assessment-and-next-plan-18281-2026-11-06.md), [seasonal pilot](SEASONAL_PILOT_REPORT-18316-2026-11-07.md), [№18,331](OFFLINE_REPAIRED_ENGINE_REPORT-18331-2026-11-07.md), [cost audit](US500_PREP_AND_COST_CALIBRATION_REPORT-18346-2026-11-07.md), [high-volatility report](HIGH_VOL_COHORTS_REPORT-18348-2026-11-07.md), and [fade/hold report](OPTION_A_VS_B_OFFLINE_REPORT-18351-2026-11-08.md).

**Codex's own process needs correction.** The sequence accumulated several bespoke runners and adapters while the central questions—execution economics, representative coverage and a falsifiable predictive mechanism—remained unresolved. Reproducing a result is necessary, but it cannot substitute for independent validation. A faster next step is to agree on the causal hypothesis and required evidence before writing another strategy variant. I also should have made the distinction between cost-inclusive production targets and the gross two-stage target explicit earlier; the new directive's mistaken premise shows that the distinction was not communicated clearly enough.

Some earlier supplied scripts assigned result counts directly instead of deriving them from replay. Those claims were rejected and documented in №18,331 and №18,346. The durable lesson is a result-writing boundary that accepts only native replay receipts, not prose, LLM estimates or constants. An LLM may propose a hypothesis, implement it and critique evidence; it must not select the claimed outcome. Independent checks should cover source identity, arithmetic and methodology, not merely reproduce the same mistaken implementation twice.

### 5.2 Why 159 episodes produced so few baseline entries

The earlier **8 entries / 159 episodes** and the earlier **13 entries in the broader cumulative ledger** are different universes. The latter includes May peer instruments outside this four-symbol experiment. They must not be treated as two estimates on the same sample.

The native `R` is the rounded half-width of a historical noise band. At each 30-minute mark, the band uses the mean absolute open-to-mark move from 14 qualifying past sessions, searched over at most 30 sessions. It is neither this week's realised range nor the size of the current spike. A violent day can therefore coexist with a small early-session `R`. Native admission also requires a band break, an acceptable spread, fresh decision and next-fill quotes, and sufficient preparation. A failed cost comparison consumes the remaining session. Later attractive prices cannot reverse that durable veto under the frozen policy.

At 24 bp per side, the approximate condition is `R / price > 0.0048 + spread / price`, before other charges. At the independent 48 bp floor, the gross 1.38R target is about 66.24 bp, while the round-trip reserve alone is about 48 bp. There is little room after spread and other costs at that boundary. This is an economic hurdle and a path-selection rule, not proof of a volatility edge. Lowering the reserve while retaining 48 bp does not redefine `R`; it may allow a later or different qualifying mark which the old session veto prevented.

Source: [`band.rs`](../crates/jiaoyi-core/src/strategy3/band.rs), [`s3_ticks.rs`](../crates/jiaoyi-core/src/research/s3_ticks.rs), and [cost reserve](../crates/jiaoyi-core/src/strategy3/mod.rs). The admission funnel and actual added/changed entries in this run are recorded in §§2–4 and the JSON.

The claim that preparation/session/censoring failures invalidate “nearly every episode” is too broad. Four of the 159 complete tick jobs have no M30 preparation; the separate failed US500 job remains outside the 159. Twenty older episodes carry historical collection-protocol flags. Same-millisecond bid/ask groups and long update intervals are different quality dimensions again. A same-millisecond pair is not proof of dropped data, and a six-second interval may reflect genuine inactivity. It does establish limits on reconstructing event order. These conditions need separate dispositions and measured counts, not one universal red flag or a blanket declaration that data are clean.

### 5.3 Mathematical limits of the goal

With exactly 20 trades, wins occur in 5-percentage-point steps. At least 73% therefore requires **15/20 = 75%**, for each strategy/account under the owner's contract. A single successful window does not establish consistent performance; overlapping rolling windows are strongly dependent. Under a simple independent-binomial approximation, 15/20 has an approximately 53%–89% Wilson 95% interval. Correlation, repeated tuning and selective stopping weaken that inference further. This is a monitoring target, not an adequate standalone research sample-size rule.

For binary gross outcomes of +1.38R and −1R, break-even before costs is `1 / 2.38 = 42.02%`. With a constant round-trip cost `k` in original-R units, expectancy is `2.38p − 1 − k`. A 73% hit rate would imply +0.7374R per trade before those costs under that simplified payoff distribution. The realised strategy also has entry-price stops, gaps, censored observations and time closes, so its actual payoff distribution must be measured.

An entry-price stop is gross-flat, not a net win. For a continuous, driftless, cost-free process with no time limit, an illustrative +1R breakeven amendment and +1.38R final target would have approximately 36.23% target hits, 50% original-stop hits and 13.77% entry-price exits. Expected gross return remains zero. This is a mathematical counterexample to the claim that exit geometry creates positive expectancy by itself; it is not a forecast or a universal ceiling for real markets. Predictive drift, conditional selection or another defensible economic advantage must pay the costs.

A 50% interim hit rate can coexist with negative expectancy. A positive PF based on a handful of equal-R trades is not a dollar-weighted portfolio return. Neither outcome supplies evidence that every asset class or account can reach the required goal.

## §6. How Codex would rebuild the codebase

### 6.1 Retain the proven Rust components and consolidate their interfaces

Keep the broker protocol and sendability/entry guards, integer price-grid arithmetic, volume limits, account ownership/reconciliation, broker SL/TP protection, actual-deal accounting and tested default trading behaviour. Move toward one versioned strategy kernel used by research, shadow operation and live execution, with explicit capability boundaries. The rebuild should proceed through small parity-checked migrations; a rewrite of all trading behaviour at once would discard useful evidence and introduce another source of drift.

| Boundary | Proposed responsibility | Existing code to retain or consolidate |
| --- | --- | --- |
| `jiaoyi-ctrader` | Transport, bounded collection, event decoding, account identity and request receipts; no alpha decisions. | Broker client, payload allowlists, tick decoder and fake-broker integration tests. |
| `jiaoyi-core` | Pure typed events, session identity, causal features, economic cost models, strategy decisions and execution state transitions. | Band/target/price-grid functions; consolidate the research/live policy definitions with explicit version IDs. |
| `jiaoyi-store` and market-data adapter | Operational order/deal/reconciliation state in Postgres; immutable, compressed market-data partitions plus a small manifest index. | Actual-deal accounting, consistent outcome snapshots and existing identity keys. |
| `crates/jiaoyi` | Live orchestration, broker acknowledgements, account/margin controls, reports and service health. | Keep account authority, pause rules and open-position protection independent of research jobs. |
| `ops/research` | One manifest-driven CLI for inventory, eligibility, replay, comparison and publication. | Reuse receipt/hash checks and useful focused tests; replace date-specific orchestration scripts gradually. |

A compact processing model is:

```text
Broker receipts -> immutable partitions + coverage manifest
                             |
Session / contract versions -> eligible dataset view
                             |
                  shared strategy kernel
                      /             \
          deterministic replay     shadow/live adapter
                      \             /
               fill / fee / outcome ledger
                             |
                comparison and review gates
```

No new orchestration framework or LLM service is required for this design. Framework adoption is justified only by a measured missing capability.

### 6.2 Make data completeness a precondition of an experiment

Store raw broker pages or a losslessly normalised representation with request bounds, symbol/account environment, instrument metadata version, bid/ask side, timestamp, source sequence where available, row counts, completion flags and content hashes. Partition market data by provider, symbol, date and stream. A shared content-addressed manifest should deduplicate overlapping preparations without copying prices into reports or Postgres telemetry tables. Retain only the source partitions required by registered studies and a bounded operational retention policy; reference counts and explicit expiry rules prevent the earlier database-volume problem from recurring.

An experiment manifest must declare its entire dependency interval: preparation lookback, signal window, permitted position lifetime, closing-quote tail, funding schedule and control exposure. Before execution, a coverage planner checks the union of all those intervals against verified cached segments. An unavailable history window becomes an explicit ineligible observation, not a zero trade and not an invitation to impute prices. The default execution command cannot collect data. A separate authorised collection step records bounded requests, retries and failures; replacement receipts preserve the failed originals and never relabel seen data as untouched validation.

Market outages cannot be permanently eliminated. What can be eliminated is starting a supposedly complete 72-hour experiment with only four hours of price coverage, silently skipping unavailable preparation, or allowing a fresh script name to bypass the spent-interval registry. The known US500 missing components remain evidence until a separately authorised repair is performed.

### 6.3 Separate broker tradability from strategy sessions

Maintain versioned calendars for (1) the CFD broker's actual dealing schedule and maintenance intervals, (2) the underlying/reference venue, and (3) the chosen strategy session. The cTrader schema already exposes schedule, timezone, holiday and swap fields; record dated snapshots and provenance instead of inventing fixed UTC offsets. Current metadata must not be silently treated as historical 2023–2025 truth.

Use IANA zones and dated holiday/early-close rules for NYSE, London and the appropriate NYMEX/CME reference contract. A Pepperstone CFD need not share every interval with its reference future. Cash opens, overnight trading, rollover breaks and strategy cutoffs must be explicit IDs. Test DST transition weeks and the weeks when US and European clocks differ. The existing NY cash-open conversion is already DST-aware; the work is to add historical schedule semantics and audit the chosen sessions, not to replace correct timezone conversion.

Preparation should request the exact eligible historical marks required by the policy. An arbitrary count such as 1,400 or 2,160 bars cannot prove those marks exist. A calendar-aware requirement prevents a legitimate closure from being mistaken for an unexplained data loss while retaining genuine gaps.

### 6.4 Preserve event ordering and carry positions across partitions

A replay clock should advance through merged market events, timer events, funding events and explicit order/acknowledgement events. Keep open positions, original R, stop versions, pending amendments, quantity and costs in state across files and trading days. Crossing a data partition must not close or reset a position. A requested time close needs an executable quote and a recorded latency rule; a missing close quote remains unresolved.

When historical bid/ask streams do not expose cross-stream sequence, mark ties as ambiguous and use a declared aggregation policy. Do not invent intermediate ticks. Distinguish ordering uncertainty, stale side, crossed quotes, unexpected transport gaps and scheduled market closures. An OHLC continuation must remain an explicitly approximate mode with ambiguity bounds or censoring, not masquerade as tick-accurate execution. Preserve broker-bracket priority and next-quote amendment semantics already tested in the existing engine.

## §7. How Codex would rebuild the strategy and cost-calibration engine

### 7.1 Separate expected execution costs, stress reserves and realised money

Give these three quantities different types and names. `ExpectedExecutionCost` is an as-of estimate used before entry. `StressReserve` is a deliberately conservative scenario. `RealisedMoney` comes from reconciled broker deals. Passing a stress scenario is not a prediction, and subtracting a stress reserve from already realised net broker P&L double-counts costs. At fixed entry and exit prices, executable bid/ask already contains spread.

Retain and extend the existing cost and outcome readers rather than building another competing ledger. `jiaoyi-store/src/trading.rs` already exposes fill latency, slippage and commission diagnostics; its `position_money` path requires closed-volume reconciliation and settled money fields. On the recorded broker convention, closing commission can represent both sides: do not mechanically subtract the opening fee again. Reconcile the convention per account and contract version against statements; preserve signed swaps, conversion fees and `moneyDigits` scaling.

For each order and closing deal, join the strategy/experiment version, account environment, instrument/contract version, decision quote, dispatch quote, side, intended quantity, order type, send/acknowledgement timestamps, requested levels, partial fills, fill prices/volumes, reject reason, actual fees, financing and quote-to-deposit conversion. Keep latency decomposition distinct from market movement. Compute signed entry and exit slippage against the appropriate executable decision/dispatch side, not against a mid-price benchmark that charges spread twice.

Estimate costs by symbol, live/demo environment, order type, size bucket and session/regime where the sample supports it. Use pooled estimates with explicit uncertainty for sparse buckets; do not force unsupported precision into every instrument. Retain both favourable and adverse slippage and include rejected/unfilled orders in execution-quality reporting. Outliers should be explained, not removed to improve apparent profitability. Freeze cost-model versions with an as-of cutoff before evaluating later trades. Report median/quantiles, sample counts, tail uncertainty and the age of evidence. Demo execution is diagnostic; it is not automatically a calibration for live fills or old historical periods.

The minimum numerical sample requirement must follow the uncertainty of the estimated quantity, not an arbitrary “enough rows” label. For the first calibration delivery, require every included observation to reconcile both legs, quantities, settled fees and conversion; missing fields remain missing. Use time-blocked holdout checks of cost prediction error. If an operational fallback reserve is needed, label it as a fallback and monitor its exceedances. Do not substitute 2 or 3.74 bp because a backtest becomes positive.

Source anchors: [cost reader](../crates/jiaoyi-store/src/trading.rs), [outcome qualification](../crates/jiaoyi-store/src/outcomes.rs), [commission research reader](../crates/jiaoyi-store/src/research.rs), [cTrader contract fields](../crates/jiaoyi-ctrader/proto/OpenApiModelMessages.proto), and the [prior calibration audit](US500_PREP_AND_COST_CALIBRATION_REPORT-18346-2026-11-07.md). This task does not query the running database or claim new empirical calibration.

### 7.2 Use a falsifiable strategy hypothesis before choosing an optimiser

No strategy can honestly be named as having the “highest probability” of success from these observations alone. My preferred research starting point is a small, economically interpretable candidate whose expected movement is large relative to measured costs, with one specified market/session and a clear reason for conditional continuation or reversal. Oil own-momentum is a candidate for such a controlled reconstruction because earlier peer and own signals often coincided; it remains unqualified. The existing results do not justify promoting either generic S3 breakout or its simple inverse.

Freeze the mechanism in one paragraph: what information is newly incorporated, which participants or trading constraints create persistence/reversal, how long the effect should last, and what observation would falsify it. A published strategy should first be replicated against its original definitions and costs before a transfer to CFD instruments is claimed. A multi-domain LLM narrative, a debate between agents or a more complex model does not replace this mechanism.

Specify distinct modules for causal features, entry eligibility, economic admission, risk distance, size, exit policy and position-management ownership. A change to one module gets a new version and an explicit ablation. Keep market risk distance independent of cost accounting. If a net target is desired, state its equation and denominator explicitly; do not silently replace a gross 1.38R target with a production cost-inclusive/partial-runner policy. A cost-aware breakeven stop needs a separately registered definition and feasibility checks; entry price alone is not breakeven after fees.

Choose the holding horizon from the proposed mechanism and transaction economics, then collect its full path. A 72-hour label is not a substitute for explaining why predictive information should survive overnight, funding, macro announcements and weekend gaps. Begin with the simplest viable session or multi-session rule; add a regime/volatility filter only if it improves a predeclared conditional forecast on later data, not merely the count of winners in this cache. Low-cost admission in this run illustrates that a retained volatility threshold does not by itself supply predictive content.

Risk sizing should use a frozen pre-trade account/risk budget, broker minimum/step, measured liquidity/cost uncertainty, margin buffer and portfolio exposure. Existing one-position live constraints and owner risk authority remain in force until separately changed. A minimum lot can still be too large for the permitted risk; the valid decision is then no entry. Kelly should not be estimated from a tiny selected sample or used to compensate for negative expectancy.

Compare each candidate with no-trade, matched own-momentum/always-long or side-balanced controls under the same decision eligibility, costs and holding limits. Report gross path returns, net returns, turnover, tail losses, exposure, unresolved paths, financing, and per-symbol/cohort results. A peer filter must show incremental value over own momentum on matched opportunities. Do not pool unrelated instruments to make one class appear qualified.

### 7.3 Make evaluation a promotion process, not a search for a target percentage

Separate development, validation and final forward evaluation chronologically. Purge/embargo overlapping labels and holding periods at split boundaries. Preparation overlap and previously inspected prices must be tracked separately from label leakage; hashing a plan does not make already observed data unseen. The 19 August–5 September 2025 holdout stays sealed in this task.

Register a bounded candidate family and evaluation budget. Report all attempted variants, including failures. Evaluate enough independent time blocks to estimate uncertainty in net expectancy and downside; dozens of overlapping trades in one event week do not create dozens of independent samples. Confidence procedures should account for dependence and repeated model selection. A class-specific 50% hit-rate screen is insufficient without costs, payoff distribution and downside constraints. If no candidate passes, finish the experiment with that result instead of issuing another automatic stop/target tweak.

Promotion requires an engineering-qualified data/execution path and separate quantitative evidence. First run the same kernel in shadow mode, then a specifically authorised demo or minimal-risk canary with frozen rules and observable rollback conditions. Reconcile every divergence between predicted decisions, submitted orders, acknowledgements and actual fills. Retain the owner's account-level latest-20 scorecard as an operational measure, but assess consistency over a longer, independently chosen horizon as well. No milestone promises the 73% outcome.

## §8. Concrete rebuild roadmap: eight phases, twelve acceptance gates

This is a proposed sequence for owner/Gemini review. The current PR delivers the offline experiment and RFC only. Each phase has a finite deliverable; later live migrations require the appropriate deployment and risk authority. “Time is Money” means resolving the most consequential uncertainty first, preserving proven fixes and stopping work that cannot change the next decision.

| Phase | Deliverable | Acceptance gates | Stop or revise when |
| --- | --- | --- | --- |
| **M0 — Contract and evidence inventory** | One versioned experiment schema, policy registry and source inventory; retain historical failures and register the gross/net target distinction. | **G1:** every result links to source, code, calendar, contract and cost versions. **G2:** baseline replay reproduces frozen decisions/fills and refusing paths before a migration is accepted. | Results lack identity, conflate variants, or require rewriting past evidence. |
| **M1 — Unified offline runner** | One local-file command for inventory, eligibility, replay, scoring and report publication; old runners become thin compatibility wrappers. | **G3:** the runner makes zero network calls in offline mode, preserves all denominators and cannot accept arbitrary result constants. | A new date cohort needs a new execution script or missing data becomes a successful zero. |
| **M2 — Market-data store and coverage planner** | Compressed immutable partitions, deduplicated manifests, explicit retention and interval-completeness checks. | **G4:** an experiment cannot claim full-path qualification without verified lookback, holding-period and closing-tail coverage; interrupted receipts remain visible. | Storage grows without a registered consumer or a gap is silently bridged. |
| **M3 — Calendar and contract registry** | Broker, reference-venue and strategy calendars with dated metadata, holidays, DST, maintenance and funding semantics. | **G5:** targeted DST/early-close/rollover fixtures match dated authoritative examples; unknown historical terms remain unknown. | A guessed fixed UTC offset or a global bar count substitutes for session evidence. |
| **M4 — Cost and actual-money calibration** | One reconciled execution ledger plus as-of expected-cost and stress models. | **G6:** quantities, fees, swaps, conversion and statement money reconcile with no double counting. **G7:** cost estimates have held-out error/uncertainty reports and explicit sparse-bucket fallbacks. | Entry-only samples are represented as round-trip costs, or demo costs are silently transferred to live. |
| **M5 — Shared execution state machine** | A deterministic kernel carries original risk, amendments and positions across partitions and restarts; live adapter uses actual acknowledgements. | **G8:** selected adversarial fixtures and recorded-event replays establish bracket priority, amend timing, gaps, partial fills, restart/reconciliation and research/live decision parity. | A file boundary closes a trade, an old plan is recalculated with today's settings, or fills precede acknowledgements under the declared model. |
| **M6 — One justified strategy experiment** | A frozen economic hypothesis, causal features, matched controls, cost specification, independent dates and bounded variants. | **G9:** split/overlap and trial-budget audit passes before opening evaluation. **G10:** net expectancy, payoff/tail metrics, controls, per-class requirements and predeclared uncertainty criteria pass; failures remain published. | A good point estimate on reused or tiny data is the only justification, or success depends on optimistic uncalibrated costs. |
| **M7 — Shadow, authorised canary and operational review** | Reconcile live-market decisions against broker receipts, then consider a separately authorised bounded deployment. | **G11:** shadow/canary execution and money reconcile within registered tolerances. **G12:** owner risk rules, monitoring, rollback and per-strategy/account scorecards work under a complete trading lifecycle. | Capital exposure outruns evidence, a service fault can bypass broker protection, or goal metrics are pooled across accounts. |

**Recommended order of attention:** complete M0 and the cost-evidence inventory first; scope M1–M3 only to the chosen candidate and its data horizon; deliver M4–M5 before quantitative promotion. The phases may overlap where inputs are independent, but G9–G12 must not be implied by CI. Three focused tests for a small policy change are useful; repeatedly running broad tests without answering the economic question is not progress.

**Human/Gemini review points:** agree on the exact cost/target definitions at M0, the first candidate and data budget before M6, and any live risk or deployment decision before M7. No decision about lower production costs, risk limits, live sizing or a 72-hour strategy has been made by this experiment. Model/agent effort cannot replace missing market evidence. These gates make the next decision reviewable and finite; they do not manufacture an edge.

**Next concrete deliverable:** a compact M0/M4 evidence inventory listing which existing receipts can support complete round-trip cost estimates, which source intervals a single chosen hypothesis needs, and the smallest authorised work needed to fill those gaps. Preserve this failed expanded-entry result as the baseline against which any later hypothesis must be compared.
