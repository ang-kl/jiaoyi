# Paired execution-cost evidence — № 17,887 §1

By Codex (GPT-6; effort not exposed). Assessment: **30 October 2026,
15:58:52 SGT**. Owner authorization: proceed with the cost-gate and quantity
assessment discussed in № 17,886.

The saved receipts show price deviations well below the slippage reserve in this sample,
but **do not validate a replacement for 24 bps per side**. No cost parameter,
entry policy, service configuration or broker order changed in this assessment.
The goal remains risk:reward 1:1.38 and at least 73% net wins over the latest
20 fully closed owned positions, separately per strategy/account. Registered
stop/target geometry is distinct from measured net realized payoff.
Goal improvement attributable to this assessment is **unmeasured**.

## Bounded source and ownership

Reused the existing private `s3-validation-cohort.json` receipt; no additional
historical dataset or database schema was created. SHA-256:
`13c514c7c0ee54afb357386a50146d7982dfcbd7d11ced6e18aa0a360772dc40`.
The receipt freezes **27 October 2026, 16:15 SGT**, read at 16:26:01 SGT.
It retains the latest 20 fully closed positions on each demo and 15 on live:
**75 positions**, all with original opening registration
`73d4f161f2fe10048d6a20a3425af94681afb663487e784f1b27eaa86e5532f7`.
Openings span 25–27 October. These are legacy observations; none belongs to the
current v6 opening hash `d54344b9d463532c3b1e2d81bd07d671ed0d2fceae9ee923f1c8988b18b2ab32`.
An exit recorded by a later program version must retain its original opening
cohort when assessing the position.

All 75 positions have one saved opening fill and one closing fill, and their
receipt volumes equal both the broker history's total opening and closing
volumes. All 75 remain in the assessment, including missing exit observations.

## What the exit reference actually measures

For each leg, signed slippage is positive when adverse. Both legs below are
normalized by that position's **opening decision price**. The adverse-pair
diagnostic is `10,000 × [max(entry_slip, 0) + max(exit_slip, 0)] / entry_price`.
It measures recorded price deviations; it excludes spreads and commissions.
It is not an all-in cost estimate or a proposed reserve.

| Exit reference | Positions | Median adverse-pair bps | Largest observed adverse-pair bps | Meaning |
|---|---:|---:|---:|---|
| Session-end decision quote | 22 | 0.224 | 2.868 | Entry and exit fills versus the corresponding executable-side quotes recorded for their decisions |
| Broker stop trigger | 44 | 1.154 | 11.334 | Entry quote deviation plus exit fill versus the recorded stop level; the exit component is trigger-to-fill gap, not quote-to-fill slippage |
| History reconciliation | 9 | Unknown | Unknown | Closing quantity and price exist, but the exit reference/slippage is absent |

The 22 quote-paired cases contain **only one observation per account/instrument**.
Their signed pair median is 0.000 bps; favorable slippage offsets adverse
slippage in that signed statistic, which is why it differs from the table.
Cross-account executions of the same signal are dependent observations.
These maxima describe this sample and are not bounds on future slippage.

For broker exits, the stored spread comes from the last quote available when
the execution event arrived. It does not establish the executable quote at
the stop trigger. The nine recovered exits have no such observation; absence
must stay unknown rather than become zero. The receipts also do not establish
historical quote freshness or uninterrupted recorder coverage.

## Reproducible checks and present projection limits

The existing receipt carries `account_id`, `position_id`,
`saved_setup.prereg_hash`, opening/closing timestamps and
`opening_and_exit_receipts`. Each fill includes `kind`, `outcome`, `sent_at`,
`answered_at`, `fill_price`, `decision_price`, `slippage`, `spread`,
`volume_cents` and `detail.reason`; broker deal provenance supplies the
independent opening/closing quantities. Pair by account and position, and
group by the original saved opening hash before summarizing.

Verified all **141 non-null price references** by reconstructing signed
slippage: a buy is `fill − decision`, a sell is `decision − fill`; closing
uses the opposite side to opening. All reconstructed values match their
stored slippage within floating-point tolerance. The remaining nine closing
references are null. Verified all 75 opening/closing quantity reconciliations.

`/trading/costs` currently aggregates dates and hashes and presents separate
entry and exit medians. These are not paired round-trip observations. A fresh
read of `/trading/orders?strategy=s3&since=2026-10-28T00:00:00Z&limit=300`
returned the limit of 300 rows, including 28 opens and 28 broker exits; trailing
amends also consume rows. This response omits original opening hashes, so it
cannot establish a complete current-registration cohort. It was not combined
with the frozen legacy sample or treated as evidence of zero missing costs.

Implementation references: [`trading::costs`](../crates/jiaoyi-store/src/trading.rs),
[`fills::slippage` and `exit_level`](../crates/jiaoyi/src/fills.rs), and
[S3 fill/broker-exit recording](../crates/jiaoyi/src/strategy3.rs).

## Quantity and the cost gate

The current per-unit model is
`C = spread + round-trip commission + 2 × 24 / 10,000 × entry_price`.
Thus 24 bps per side contributes **48 bps of entry price round trip**.
The initial stop distance is `R`; admission rejects `C >= R`.
For any positive quantity `q`, `(q × C) / (q × R) = C / R`.
Reducing quantity can reduce cash risk or margin demand but cannot rescue this
per-unit cost rejection. Broker minimum quantity and step still apply.
This algebra describes the implemented model; it does not assert that all
real-world commissions or market impact are perfectly linear in size.

## Evidence-gated follow-up

The sample was selected from already observed closed outcomes over a short
period, excludes still-open positions, and uses a legacy strategy registration.
The sparse quote-paired groups and mixed exit references do not validate exit
tails or a cost allowance for every asset class. The 24-bps reserve appears
conservative relative to these recorded deviations, which warrants calibration
work; it does not establish a safe replacement value.

Before proposing a new registered allowance, obtain a bounded, complete
opening-hash cohort from the existing order/fill/deal records. Retain all
positions and quantity reconciliation, keep decision-quote slippage separate
from stop-trigger gaps and unknown observations, account for spread and actual
charges without double counting, and retain shared-signal dependence. Assess
the candidate on evidence distinct from that used to choose it under the
existing registration process. This note creates no new endpoint, telemetry
collection, parameter search, automatic activation or financial PASS.
