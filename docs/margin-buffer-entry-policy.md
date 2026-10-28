# Margin-buffer admission — 28 October 2026

Source: owner approval acknowledged at chat №17,666 §1, following the comparison
in №17,665 §§1–4 (28-10-2026 16:21 SGT). By Codex (GPT-6), effort not exposed.

## Activation boundary

The owner selected a margin buffer for new entries and retained the existing
one-position live rule until Strategy 2 has a tested basket-execution plan.
The minimum percentage was not specified. The implementation is prepared for
review; **no live value, merge or deployment is authorized by test percentages**.
The separately requested owner value is required before release. A missing
policy value refuses an entry; consequently this change must not be deployed
without the reviewed activation setting. Values 200% and 250% in tests are
synthetic fixtures, not production defaults or measured account requirements.

## Proposed entry rule

A fresh, complete assessment must satisfy both conditions:

1. Projected equity / projected gross margin ×100 is at least the owner's
   `trading.margin_level_floor_pct` for the account.
2. After the additional loss reserve to existing stops, pending-entry stops
   and the candidate's original stop, the resulting margin level remains
   strictly above the highest positive warning threshold returned by that
   account's broker. This threshold is read; it is never changed.

The gate reads an account-scoped trader record, the whole broker book including
protection orders, unrealized P&L, the symbol catalogue, margin warnings and
exact-volume expected margins. Each monetary response uses its own precision.
Gross buy and sell exposures are reserved without assuming hedge discounts;
current allocated position margins provide a second conservative baseline.
Existing manual/external positions and pending entries are included in capacity
and stop reserves, without adopting, cancelling, resizing or managing them.
A protective order is not treated as another opening order. A pending future
position ID with an exactly matched zero-P&L row may be reserved without
adoption; nonzero or ambiguous unmatched P&L is not accepted as complete equity.
Pending remainders must be explicit. Linked increases to an existing position,
unsupported GSL estimates, unknown protection or unavailable conversions refuse
the candidate. Deposit-currency exposure and the candidate's verified quote
conversion are supported; other quote currencies require fresh verified
conversion evidence and remain unavailable in this bounded implementation.

The assessment reserves current broker estimates for pending fills; future
trigger prices, gaps, changes in leverage or conversion, and unquoted closing
costs can change the outcome. The stop projection is not a guaranteed maximum
loss or a guarantee against margin call/stop-out. Bonuses are not counted as
cash equity. No account balance or monetary exposure is exposed on public routes.

New-entry admission is serialized per database/account. Unknown unresolved
program entries refuse another entry while their margin reservation is unknown.
The final broker book and trader record must match the initial read; changed or
unreadable sources refuse the candidate. The additional read phase is bounded
at five seconds. Exits do not take the entry lock or run margin admission.
This does not establish an execution-latency SLA for the existing actor loop.

## Existing boundaries

Every frame still uses the one gate. Existing owned stop amendments and closes
bypass this new entry check; ownership, stop improvement and TP preservation
remain enforced. No pause key, risk percentage, live minimum, open-risk budget,
SL/TP geometry, alpha registration or live account is changed. There is no
basket exception and no Strategy 2 activation. The agreed latest-20 benchmark
and 1:1.38 risk:reward reference remain unchanged. Mixed historical results
must not be presented as evidence that the new admission policy improves returns.

## Evidence and delivery

One compact assessment is appended to the existing decision ledger per actual
candidate. `/operations/reports.margin_admission` exposes only typed policy and
percentage fields for allowlisted accounts, with the last assessment time;
missing values remain unconfigured/unknown. It is not a current margin certificate.
No raw quote archive, new telemetry table, service, model polling or paid API is
introduced. Required Rust, trading-image and engineering-image CI must pass on
the final head, followed by actual broker-source and deployment read-back before
claiming activation or broker acceptance. Local scripted-broker tests establish
implementation behavior; they do not establish live broker values or profitability.
