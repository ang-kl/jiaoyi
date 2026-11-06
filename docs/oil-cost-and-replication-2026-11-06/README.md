# Oil cost evidence and frozen replication plan

№18,283 §§1–6 · 06-11-2026 14:56:16 SGT (+08:00)
By Codex (GPT-6); effort setting not exposed.

## §1. TLDR

**Do not lower the cost assumption or claim improved wins.** The latest views
contain 263 filled opens but only 13 measured entry costs, all from demos.
Oil contributes five measurements, all sells on one day. All five have linked
exit records, but the views cannot establish complete, paired round-trip costs.
The available evidence therefore does not qualify a calibrated cost estimate.

The next oil development experiment is frozen: **20 symbol/date episodes across
10 dates**, with stronger per-symbol and multi-week gates. No new market-data
jobs or validation runs have been started in this delivery. The original
73% / 1:1.38 goal and interim 50% goal remain unproven.

Files: [cost assessment](cost-assessment.json), [frozen plan](oil-replication-plan.json),
[plan SHA256](oil-replication-plan.sha256), [verification receipt](verification.json).
Earlier tested results: [№18,280](../full-exit-research-2026-11-06/README.md).

## §2. What was checked

Nine read-only snapshots were retrieved on 6 November 2026 at
14:42:23–24 SGT: eight newest-500 order views covering S1/S3 on four accounts,
plus the aggregate cost view. They contain 1,114 unique order records. No page
reaches its 500-row limit, but that does not certify an unbiased historical census.

| Scope | Filled opens | Measured entry costs | Qualified round trips in this review |
| --- | ---: | ---: | ---: |
| All accounts | 263 | 13 | 0 |
| Live | 22 | 0 | 0 |
| Demo | 241 | 13 | 0 |
| Oil, subset of above | 31 | 5 | 0 |
| Metals, subset of above | 22 | 0 | 0 |

Entry attribution arithmetic was independently recalculated, retaining signed
favourable as well as adverse slippage. All 13 records agree. This verifies
arithmetic, not the representativeness of the sample. The newest measured
maximum adverse entry dispatch-to-fill slippage is **4.436bp**, on ETHUSD.
The earlier 3.74bp sensitivity is not an observed upper bound or a calibrated
live-cost allowance. Neither is 24bp a proven empirical bound; it remains the
existing stress assumption.

The five oil measurements are all from 3 November 2026, across three demos.
Their native quantities are 500, 800, 8,200, 10,800 and 22,000; the proposed
research quantity is 100. None establishes representative live execution or
historical July/August 2025 costs. Metals have no measured entries in these views.

## §3. Why linked exits are insufficient

All five measured oil entries have linked exits: two session-end market closes
and three broker exits. However, these views lack matching final-dispatch exit
benchmarks and complete commission, swap and conversion-fee lifecycles. The
zero qualified count is a limit of this review, not a claim that the broker
has no usable cost records elsewhere.

Source inspection found that the aggregate
[`costs()` query](../../crates/jiaoyi-store/src/trading.rs) sums absolute stored
deal commissions by position. Its `commission_round_trip_per_unit_median`
does not require a settled, quantity-complete closing lifecycle and excludes
funding/conversion costs. An aggregate zero therefore cannot certify zero fees.
Mixed opening/closing representations can make this unsuitable for calibration;
actual overstatement on these positions is **Not Verifiable** from these views.

The separate `position_money()` accounting path already requires matching
opening/closing quantities and uses explicit closing-deal commissions, swap and
conversion fees. It deliberately avoids charging opening commission again.
This delivery leaves that accounting intact. The finding is recorded as a
qualification gap, not used to justify an unverified production accounting fix.

To qualify costs, reconcile broker opening/closing quantities, money precision,
commission semantics, swap/conversion charges and paired executable benchmarks
for representative symbols, sides, quantities, sessions and account environments.
If a field is unavailable, retain unknown costs rather than manufacture zero.

## §4. Frozen next oil experiment

Plan SHA256:
`647bb800945e47130293006667626289449b7ff53a017c876fa2e2cd6a54a1ea`.
Committed before any new collection as `900eb0f`.

Ten weekdays: 21, 23, 25, 28, 30 July and 1, 4, 6, 8, 11 August 2025.
Each includes SpotCrude and SpotBrent, 07:00–11:00 UTC, demo 49236051,
native quantity 100. They precede the separate validation dates and exclude
the earlier two development dates. Global prior exposure is unknown; these
are additional development dates, not a pristine holdout.

The primary candidate remains peer continuation. Own momentum and always-long
are matched controls, not replacements selected after outcomes. Rules retain
the native Q=1.38 target planner, original-R stop, partial/runner handling,
30-second trailing behaviour and quote freshness/coverage requirements.
The original collection baseline is separately verified before candidate analysis.

Primary geometry and scoring retain 24bp per side. Realised 2/3.74bp scenarios
keep the same entries and geometry and remain diagnostic. Unknown additional
commission is explicitly modelled as zero for the scenario, not certified net.

The primary trailing candidate must satisfy all of these development gates:

- At least five closed trades **per symbol**, spanning at least three distinct
  ISO weeks per symbol; at least ten closed trades in total.
- At least 50% scenario net wins, positive mean net R and profit factor above
  one for both the class and each symbol; positive price-unit sum per symbol.
- No missing or censored episodes; preserve the complete planned episode set.
- Report matched control differences. Identical peer/own decisions demonstrate
  no incremental peer value, even if the numerical performance screen passes.

These are preliminary screening gates, not statistical proof of an edge. Costs,
independent validation and prospective shadow evidence still govern promotion.
No validation is opened if development fails or remains insufficient.

## §5. Six checkpoints and the next implementation step

1. **Evidence integrity:** verify snapshot, source-code and frozen-plan hashes.
2. **Collection preflight:** confirm historical availability, exact broker
   symbol/quantity/grid, and no competing active job. Current contracts do not
   certify historical specifications.
3. **Bounded collection:** one active job, at most two submissions per invocation,
   twenty total; no automatic retry or replacement dates. Combined bid/ask cap
   200,000 ticks and 120 pages per job; 32MB per source, 640MB total.
4. **Replay integrity:** verify original study and fresh executable quotes,
   retain gaps/censoring, then use the unchanged native full-exit engine.
5. **Development decision:** apply every frozen gate and matched control;
   reject or mark insufficient without retuning or collecting until it wins.
6. **Qualification:** only after passing development, lock the candidate and
   evaluate separate dates; require representative costs and prospective shadow
   evidence before any live promotion. No live promotion is authorised by a score.

**Remaining implementation:** the existing `full_exit_research.py` pins the old
plan and original 40-file manifest. A minimal adapter must accept this separately
frozen plan and enforce its source/budget checks without weakening the old guard.
That adapter and the 20 collection jobs are not implemented or queued here.
Historical availability is still unverified. The next concrete deliverable is
that bounded adapter plus a two-episode preflight, stopping on source failure.

Index and metals rules are not retuned in this oil experiment. Index changes
need an independently justified predictive mechanism; metals need feasibility
and cost evidence before exit optimisation.

## §6. Reproduction and limits

The offline [assessment script](../../ops/research/cost_evidence_assessment.py)
accepts the nine saved snapshots and manifest, checks SHA256 and size bounds,
and writes the derived assessment. It never calls the broker or fits costs.

```sh
python3 ops/research/cost_evidence_assessment.py \
  --snapshots /path/to/nine-snapshots \
  --output /tmp/cost-assessment.json
python3 -m unittest discover -s ops/research -p 'test_*.py'
```

The published JSON includes source timestamps/hashes, 13 retained attributed
entries and their linked exits. Raw account snapshots are not duplicated in Git.
Exact independent raw reproduction requires those source files; derived JSON
inspection alone is not a complete source reproduction. No ZIP, additional
production telemetry, model API call, trading-rule change or worker-policy
change is introduced. CI/merge status is recorded on the linked PR after checks.

Post-review verification: the script now also accepts the broker's valid nullable
decision benchmark while retaining dispatch/fill evidence. One regression failed
before the fix and passed after; the suite now has 23 passes and one optional
skip. Replaying the same snapshots reproduces every assessment field except the
script hash. The frozen assessment retains its original code hash from commit
`1d8819f`; the verification receipt records both hashes. The plan is unchanged.
