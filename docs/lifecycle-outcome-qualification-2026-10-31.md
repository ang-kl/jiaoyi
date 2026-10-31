# Current lifecycle authority and shared outcome qualification

Owner instruction: fix the three reviewed amendments. Trace starts at
№18,018 §1, 31 October 2026, 14:10 SGT; by <Codex (GPT-6), effort: not exposed>.
The separate coordinator repair is PR144. This document covers the two
application boundaries; merge and deployment evidence belong in the PR closure.

## Position management

Historical ownership proves that an opening once belonged to Jiaoyi. It does
not authorise management of a later external reversal, addition or reopening
under the same broker position ID. Adoption and the common amend/close gate now
use positively correlated opening orders, actual execution fills and reductions
to reconstruct the owned residual. They check its symbol, original side and
quantity against the fresh broker position. A label, requested quantity, current
direction or current position snapshot alone does not grant authority.

Valid partial fills and reductions remain manageable when their continuity is
proved. Mixed positions are not divided into a presumed owned fraction. S3 plans
stay bound to the original opening key. Reconciliation snapshots do not become
fabricated incremental fills; duplicate receipt/history deal IDs count once.
Unconfirmed openings and TP1 writes retain their existing no-resend controls.
Session-end closes retain their existing reconcile-and-retry policy; this repair
does not broaden that authority.

After restart, an execution-stream gap or incomplete evidence, a bounded
read-only lane obtains orders and deals for the affected position. It runs
alongside the existing management loop, with bounded queue, timeout and retry
interval. It neither resends an opening nor introduces another trading writer.
While current exposure remains unqualified, both strategies defer new entries
through their existing unknown-risk path. Required evidence-read/write errors
terminate the affected actor, including its nested read pump and trailing path.
Truncated responses and ambiguous shortened order keys remain unverified. A
position observed absent loses its old management authority permanently; this
marker records no financial close. An executed foreign opening, or disagreement
between executed orders and returned deals, cannot certify recovery. Broker
SL/TP stay in place while authority is unresolved. Tests cover recovery while
historical preparation is held and valid subsequent exit handling.

The broker does not offer an atomic compare-and-send ownership transaction.
Fresh reconciliation reduces the stale-state risk but does not remove the race
between that read and a later external mutation. No race-free claim is made.

## Completed outcomes

Goals and the shared closed-outcome reader now use the same per-position facts
and monetary rows from one repeatable-read snapshot. Performance, drawdown/limit
assessment and existing Kelly cohort selection consequently consume the same
qualified completed outcomes. They retain their existing strategy/account/hash
filters, original risk interpretation and thresholds.

Qualification requires owned opening provenance, terminal quantity completeness,
matching deal identities with sufficient quantities, supported fill status and
complete finite monetary data. Local close receipts, broker exits, named opening
receipts and unclassified execution events are checked. An equal aggregate total
cannot conceal a contradictory individual deal. A history revision may make an
outcome incomplete; that cannot silently clear a previous risk alert. Later
valid evidence can restore normal qualification.

Cash/deal records are retained. Incomplete positions contribute an unresolved
count rather than a fabricated return, win or zero. Invalid monetary precision
and nonfinite R remain unmeasured. A partial trade is not counted as a completed
win simply because its available opening and closing subsets happen to balance.

## Acceptance and remaining limits

Targeted broker-gate, real database, actor and coordinator fixtures exercise the
reviewed failure families and positive controls. A private dated projection of
the three current owned broker positions also qualifies under the new authority
reader. That projection is a saved-source check, not a production database or
live recovery test. Deployment must separately verify source identity, a single
active trading instance, all actor loops, owned/external book separation and
unchanged broker protection.

The tests cannot establish that every future fault is impossible. These repairs
also do not establish a profitable strategy. The goal remains risk:reward 1:1.38
and at least 73% net wins over the latest 20 qualified closed positions per
strategy/account (at least 15 wins in a 20-trade window). The inquiry's missing
original monetary-risk evidence, complete evaluator/control composition,
information-time inputs and prospective study design remain open. No new model,
strategy, risk limit, owner pause, allowance or worker mutation deadline is
introduced by this repair.
