# Backtest report for independent AI review

№18,308 · §§1–9 · 07-11-2026 08:21 SGT (+08:00).
By Codex (GPT-6), effort setting not exposed.

## §1. TLDR: what has actually been tested

**Yes: two retained historical BTC/NatGas datasets were replayed. Both produced
zero simulated trades because the first signal failed the registered cost gate.
Their win rate and profit factor are unavailable, not zero. No improvement in
financial performance has been demonstrated.**

**No: the newly implemented production-rule oil adapter has not yet been run
as a genuine historical oil performance backtest.** Its two executable runs
used artificial prices. They verify software behaviour, not an oil edge.

There is also an **earlier, separate historical oil momentum experiment** with
20 episodes, 9 simulated closes and 3 wins: 33.33% net wins, mean −0.7256R,
PF(R) 0.1964. It failed. Those saved results remain relevant, but that experiment
used different entry rules and a shorter window than production S3. It was not
rerun from raw oil data in the latest implementation task.

| Evidence | What happened | What it establishes |
|---|---|---|
| Latest retained BTC/NatGas market replays | Exact equality with two previously saved complete studies; no entries | Compatibility and cost-veto behaviour on those dates |
| New Crude-long/Brent-short synthetic CLI runs | 24 scheduled marks; artificial exit paths and quality flags | Execution of the adapter and selected deterministic mechanics |
| Three new regressions, seven S3 tick tests total | Passed; core all-target Clippy passed | The tested code conditions work under their fixtures |
| PR #175 repository CI | `check`, `docker`, `engineering` passed; automated review found no issues | Build/test/review status, not a financial edge |
| Earlier oil experiment | Negative expectancy and insufficient qualification | Reject that tested candidate under its assumptions |
| Financial targets | Zero newly qualified account trades | Interim 50% and original 73% / 1:1.38 goals remain unverified |

The report consolidates existing evidence; it does not claim a new collection
or new performance run during report preparation. Its scope is these research
experiments, not a fresh audit of all accounts or all 20 watched instruments.

## §2. Exact historical replays from the latest implementation

Source: [retained market replay outputs](../docs/s3-oil-replay-18307-2026-11-07/verification.json),
field `retained_market_replays`. Account context: demo 49236051. Dates are
**6 October 2025**, not the current November 2026 market.

| Measure | BTCUSD | NatGas |
|---|---:|---:|
| UTC requested interval | 00:00–04:00 | 07:00–11:00 |
| SGT interval, same date | 08:00–12:00 | 15:00–19:00 |
| Bid / ask updates | 30,037 / 30,032 | 816 / 879 |
| Reconstructed snapshots | 30,052 | 1,229 |
| Decision marks in requested interval | 7 | 7 |
| First signal | Buy at 00:30:00.253 UTC | Buy at 07:30:00.483 UTC |
| Original R, price units | 148.86 | 0.005 |
| Cost reserve C, price units | 608.983776 | 0.0197536 |
| C / R | 4.090983 | 3.950720 |
| First-mark outcome | `cost_ge_r_session_veto` | `cost_ge_r_session_veto` |
| Remaining six marks | `session_cost_veto` | `session_cost_veto` |
| Simulated entries / closed trades | 0 / 0 | 0 / 0 |
| Net win rate / PF / expectancy | Not measurable | Not measurable |

R and C above are **instrument price distances**, not dollars lost by the
account. No trade was opened, so the table does not report a realised loss.
The requested native volume was 1 for BTC and 10,000 for NatGas; these are
broker-native quantity units and must not be presented as lots without mapping.

Both runs retained 24 basis points of slippage reserve per side, zero assumed
round-trip commission and spread assumptions of 15.0 for BTC / 0.004 for NatGas.
These are frozen research assumptions, not newly measured costs. The formula
is prepared spread + round-trip commission + two-sided slippage reserve at the
executable price. Once C >= R, the registered rule vetoes the session.

The six later skips do not independently evaluate six new signals. Likewise,
two four-hour windows do not establish that these strategies never trade on
other dates, and their zero-entry results cannot measure predictive accuracy.
They do not exercise the newly added oil dispatch recheck: the original gate
already refuses entry before that step.

Coverage remains limited. BTC has two long intervals and 30,017 grouped
same-millisecond updates. NatGas has 561 long intervals, 431 stale-side
snapshots and 466 same-millisecond groups. These counters flag uncertainty;
they do not prove the broker lost ticks. Exact output reproduction does not
convert an incomplete historical path into continuous executable evidence.

## §3. Synthetic oil verification: do not count these as market wins

Source: the same [verification JSON](../docs/s3-oil-replay-18307-2026-11-07/verification.json),
field `synthetic_cli`.

| Fixture | Direction | Requested session | Quotes supplied | Trailing scenario result | Qualified net result |
|---|---|---|---:|---:|---|
| SpotCrude | Long | 07:00–20:00 UTC | 5 bid + 5 ask updates | +0.9772R | Unavailable |
| SpotBrent | Short | 07:00–20:00 UTC | 5 bid + 5 ask updates | +1.0228R | Unavailable |

**Clarification of the earlier “full-session CLI” wording:** the requested
session spans 13 hours and generates 24 marks, but the artificial quotes only
cover 07:30:00–07:30:41. These are not continuous 13-hour market datasets.
The trailing arms close under the constructed path; the fixed-stop arms remain
censored, with scenario net R unavailable. Both paths retain gap flags.
Neither two positive synthetic results nor 24 generated marks is a win-rate
sample. The evidence sets `financial_pass: false` and
`account_admission: not_reconstructed`.

The three new tests cover: preparation evidence and causal mark behaviour;
C < R at decision followed by C >= R at dispatch causing a session veto; and
long/short partial exits with quality flags and censored-path handling. They
are focused subsets of the specification's acceptance groups, not proof that
all account-state, restart, freshness and acknowledgement cases are finished.

## §4. Earlier historical oil experiment: failure retained

Source: [original report](../docs/oil-replication-results-2026-11-06/README.md),
[full results](../docs/oil-replication-results-2026-11-06/results.json) and
[frozen plan](../docs/oil-replication-v2-2026-11-06.json).

Twenty symbol/date episodes covered ten development dates between 21 July and
11 August 2025, two oil symbols, 07:00–11:00 UTC, demo 49236051 and fixed native
quantity 100. The primary was first-hour own momentum, original-R trailing and
24 bp/side. Eleven episodes were vetoed by costs; nine closed.

| Primary result | Closes | Wins | Net win rate | Mean net R | PF(R) |
|---|---:|---:|---:|---:|---:|
| SpotCrude | 6 | 2 | 33.33% | −0.556933 | 0.309624 |
| SpotBrent | 3 | 1 | 33.33% | −1.062931 | 0.029729 |
| Combined class | 9 | 3 | 33.33% | −0.725599 | 0.196433 |

The observed win-rate shortfall to 50% is 16.67 percentage points for each
symbol and the class. This is not qualified account-goal progress. PF here is
computed in R units, not pooled account-dollar PF. Every primary closed path
has a coverage flag; Brent also lacks the required number of closes/weeks.
The formal evidence insufficiency does not erase the observed negative returns.

Reducing the **reporting sensitivity** to 2 bp/side did not rescue trailing:
4/9 wins, mean −0.0911R, PF 0.8048. The fixed-stop 2 bp aggregate was marginally
positive but still below 50% class wins and negative for Brent. Those arms
retain the primary admission/geometry; they are not new fully recalibrated
strategies. Selecting one after seeing results would not qualify a replacement.

This candidate used first-hour direction/range and a four-hour experiment.
Production S3 uses historical noise bands, repeated half-hour decisions and a
13-hour registered oil session. The candidate's failure is evidence against
that candidate; it is not an empirical test of every production-S3 behaviour.
The original raw oil files are missing from the restored environment. The
committed derived results can be inspected, but a fresh raw-source rerun is
not claimed by this report.

## §5. What PR #175 changed, and why

[PR #175](https://github.com/ang-kl/jiaoyi/pull/175) merged as
`e8fdf1b448876aa189d2853b2cf44f2a398c803b`; reviewed head
`8a049b661b2143dad02c333e7d8edcd80abbca36`.

The new [prepared adapter](../crates/jiaoyi-core/src/research/production_s3.rs)
and [offline CLI](../crates/jiaoyi-core/examples/production_s3_replay.rs):

- Require caller-supplied preparation reference/time, positive opening-bar
  volume and cost values consistent with the request.
- Restrict the new route to the two oil symbols and current S3; preserve
  legacy research paths. Apply the production 45-day historical view.
- Recheck C using the next executable quote as a dispatch proxy. If C >= R,
  refuse entry and veto subsequent marks without refitting TP/SL geometry.
- Reuse the existing full-exit engine, bounds and native quality flags;
  verify typed input checksum and emit a replay-algorithm fingerprint.

The HTTP route refuses this prepared mode before collection because it cannot
carry the required evidence. The new capability is **offline**, not a deployed
automated oil research collector. Production's actual entry logic already had
the final cost guard; the correction is to the research simulation.

## §6. What is still missing before a defensible oil result

1. A retained or newly authorised bounded **development** dataset containing
   the full session/tail and the required historical preparation inputs.
2. Causal M30/opening-volume evidence and preparation/cost provenance. The
   wrapper checks declared fields but does not authenticate a broker receipt.
3. Historical contract and account state if claiming account-level behaviour:
   equity, margin, pauses, pending positions, Kelly cohort and ownership.
4. Remaining dispatch/restart/client-freshness/acknowledgement parity cases.
   The next-quote assumption is not measured network or broker execution.
5. A frozen development evaluation and then separate validation/shadow evidence.
   Recent November 2026 stored oil ticks remain beyond the development seal;
   their availability inventory was not a performance evaluation.

The old twenty-job allowance remains spent. Missing files do not reset it.
No lower cost, larger quantity, profitable date or weakened gate is selected
to manufacture a pass. These limitations concern the claims that can be made;
they do not mean every software task must wait for a new market sample.

## §7. Proposed next actions and stop conditions

First, independently review the implementation against the production actor
and v7 registration. Make only concrete, fixture-backed parity repairs.
Second, resolve the exact source/preparation gaps with a bounded acquisition
proposal; distinguish an assumed-cost scenario from an account reconstruction.
Third, freeze dates, symbols, costs, primary exit arm, metrics and refusal
rules before evaluating any new oil outcomes.

Record all skipped/missing/censored episodes. Require at least the existing
pilot floors and positive expectancy/PF alongside 50% net wins for each symbol
and class; small samples alone do not establish future win probability. Keep
the separate validation closed until its conditions are met. A failed candidate
is retained as failed, rather than retried on progressively friendlier dates.

Stop financial promotion when provenance, coverage, per-symbol performance or
account authority is insufficient. Software CI is necessary for safe delivery
but does not substitute for an economic edge. Original goals remain 73% net
wins and reference risk:reward 1:1.38 over the latest 20 owned closes per
strategy/account; none of this report certifies that goal.

## §8. Reproducibility and review package

- [Compact evidence manifest](BACKTEST_EVIDENCE-18308-2026-11-07.json): source
  hashes, exact ratios, requests and separated historical/synthetic summaries.
- [Instructions for the other AI](AI_REVIEW_BACKTEST-18308-2026-11-07.md).
- [Original specification](../docs/s3-oil-replay-spec-18306-2026-11-07.md).
- [Implementation notes and known limits](../docs/s3-oil-replay-18307-2026-11-07/README.md).
- [Production S3](../crates/jiaoyi/src/strategy3.rs),
  [v7 registration](../docs/preregistration-2026-11-01-strategy3-v7.md),
  [native replay/tests](../crates/jiaoyi-core/src/research/s3_ticks.rs),
  [exit engine](../crates/jiaoyi-core/src/research/exit_replay.rs).

Retained raw BTC/NatGas jobs are outside Git; the manifest identifies their
hashes and local paths. A reviewer with only GitHub can review code and saved
outputs, but cannot independently rerun the raw market data from these report
files alone. No private credential or raw corpus is embedded in this package.

## §9. Invariants and conclusion

Passed for this report: distinct dataset identities, serial/paragraph/SGT
attribution, source links and arithmetic checks, preservation of failed evidence.
Not verifiable: historical oil profitability under the new adapter, full
production/account fidelity, 50% interim and original account goals.
Newly qualified trades: **0**. No new performance experiment during this report.

**The backtest infrastructure progressed. A winning strategy has not yet been
demonstrated.** The next reviewer should challenge the remaining source and
model assumptions, not count synthetic successes or zero-entry replays as wins.
