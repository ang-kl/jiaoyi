# Continuous assessment and operating milestones

Recorded 26 October 2026, 10:06 UTC / 18:06 SGT, from the owner's request to
appoint a continuous main agent and sub-agents, report milestones and human
decisions, and continue while the owner sleeps in SGT. The owner described
market coverage as 22/6. Each instrument still follows its registered broker
session; this does not authorize trading closed sessions.

## What runs continuously

The intended persistent component is a read-only assessment supervisor inside
the existing Railway jiaoyi service. It evaluates results and records reports
through the existing database and service. It is not a second trading executor
or an LLM that can autonomously rewrite and deploy code.

The root agent coordinates engineering during active task execution; child
agents handle bounded assignments. Chat agents are not guaranteed to keep
running between turns, after task termination or through environment shutdown.
Railway processes also depend on service, database and broker availability.
Restart recovery, current assessment age and failures must be observable.

At this document's creation, persistent supervision, goal metrics and fresher
deal polling are **in progress, not verified deployed**. Their completion needs
the evidence below. Continue independent authorized work while owner decisions
are pending; do not turn an unanswered question into approval.

## Authority and measurement

- Preserve the [charter](factory-charter.md), registered strategy rules, existing
  order gate, ownership, stops, targets and sizing. Only the owner changes a
  pause or raises risk. Alerts do not pause or liquidate positions. An owner
  pause preserves every existing exit.
- Assess each strategy on each account separately. Use the latest 20 fully
  closed owned positions, with authoritative broker net costs and original
  recorded risk. Partial exits remain part of one position. Never fill missing
  cost or ownership evidence with zero or a guess.
- The 73% win target requires at least **15 net-positive positions out of 20**
  (75%); break-even positions remain in the denominator. Fewer than 20 is
  insufficient evidence. Missing or newer unreconciled closing evidence must
  prevent a passing verdict on an older convenient cohort.
- Report realized net-R payoff against the **1.38 reference** separately from
  planned risk:reward and SL/TP geometry. This reference does not change a
  registered target. Give explicit undefined/no-loss states where a ratio
  cannot be calculated; PF is a separate ratio, not a win percentage.
- Retain successive window identities and their results. One passing window
  does not establish consistency, and overlapping windows are not independent
  trials. No required number of consecutive passing windows has been invented.
  Existing strategy pre-registration gates remain in force.

## Assignments

| Owner | Bounded assignment |
|---|---|
| Root agent | Integration, actual check results, PR/CI release, exact Railway revision, live verification, owner decision queue and closure. |
| `goal_metrics` | Per-strategy/account latest-20 metrics, cohort identity and missing-evidence safeguards. |
| `runtime_supervisor` | Persistent assessment, deduplicated durable reports and failures through the existing service. |
| `history_freshness` | Timely authoritative deal refresh through the existing history worker, without repeatedly downloading all ticks. |
| `milestones_contract` | This milestone contract, journal and independent source review; no trading authority. |
| `strategy1_firstfill_audit` | Read-only trigger/admission audit; O-4 stays open until a natural first fill is checked. |

An assignment is not evidence of completion. Review actual changed paths and
failure controls before adopting an agent's claim. Only applicable proven
lessons from bot-trade are carried forward, with their source and check.

## Milestones and exit evidence

| Milestone | Current disposition | Required result and receipt |
|---|---|---|
| M1 — trustworthy goal measurement | In progress | Correct ordered latest-20 results on each account/strategy; 15/20 threshold, cost/ownership/partial-fill controls, explicit missing/stale states, actual executed regression results. |
| M2 — persistent supervision and reports | In progress | The existing service assesses without an active chat turn, records window/status milestones durably, survives restart without duplicate milestones, and exposes report age/failure through a read-only operations view. Exact deployed revision and real read-back required. |
| M3 — fresh broker history | In progress | New authoritative closing deals reach assessment promptly through the existing worker; long tick/probe work does not silently starve deal freshness. Confirm actual new-deal storage, replay/dedup and recovery, not merely a healthy process. |
| M4 — margin admission with existing pre-order/SL checks | Margin buffer selected; numeric floor pending (28 Oct, №17,666) | Preserve the existing decided order/SL/TP and account rules. Establish broker margin precision, account aggregation and GSL compatibility. Owner selects the margin floor or capacity-only policy before applying a new rule. Verify refusals and exits under delayed/unknown inputs; no silent resizing. |
| M5 — remaining handover contracts | Open/deferred | Natural Strategy 1 first-fill audit; measured-cost-based, pre-registered C4 blackout; C5 Strategy 2 only after the owner resolves basket admission and its bounded trial is registered. Never force a trade or resume paused research to fill a checklist. |
| M6 — goal evidence and closure | Open | Current, cost-complete evidence for every required strategy/account, retained successive-window results and the applicable registered acceptance gates. State gaps and owner decisions explicitly. No guarantee of economic success or automatic trading pause when a numeric window passes. |

Release evidence includes unchanged required CI gates, exact merged/deployed
SHA, service readiness and the requested live behavior. Keep software success,
deployed behavior and economic results as separate claims.

## Reporting while the owner is asleep

The supervisor's operational target is continuous assessment while the service
runs, including unattended SGT hours. Record UTC source times plus SGT display
times. Reports identify account, strategy, window, data age, result, missing
evidence, milestone and action owner. Closed markets are not a stale-quote
incident merely because they have no quotes; use registered session context.

Persist a report when the measured window/status changes and at meaningful
sample milestones (5, 10 and 20 closed positions), rather than repeating the
same alert each poll. The operations page also shows the latest 100 non-heartbeat
events, so an overnight milestone remains visible after subsequent heartbeats;
the full record stays in the existing decision history. Record new failures, missing/stale evidence and recovery
with their last-success time. Routine owner decisions remain queued until the
owner's next session; independent engineering continues.

Record critical account/coverage/data failures immediately in the existing
database, health/operations reports and logs. There is **no configured push
notification destination or guaranteed wake-up delivery**. Reports visible on
an endpoint are not messages delivered to a sleeping human. Existing exits
remain active; an unanswered critical alert does not authorize automatic pause,
risk changes, account activation or forced liquidation.

The old 07:35 SGT report is bound to Claude's session in the handover. Its
transfer/delivery here is unverified. Do not claim that schedule is working in
Codex or create a duplicate report sender. This work provides persistent service
reports; any external delivery or schedule transfer needs an actual supported
destination and delivery evidence.

## Owner decisions and handover disposition

Pending decisions remain explicit: the affected demo 5306502 new-entry pause;
the proposed margin floor/capacity-only policy; Strategy 2's 2–4-position basket
versus the live one-minimum-trade rule; and the old daily report's destination.
The earlier keep-running instruction remains active until the owner changes it.

O-1 broker-exit recording and O-2/C3 are verified completed in the
[journal](../journal/2026-10-26.md). O-3 report routing is unverified; O-4 first
Strategy 1 fill and O-5/C4 remain open; O-6/C5 is not built; O-7's 50-trade
size review is conditional and not permission to increase size; O-8 research
R1/R2 remains owner-paused. The margin incident is not declared resolved by
backoff alone. The whole handover and financial goal remain open.


## 28 October 2026 margin/basket disposition

Owner source №17,666 §1 supersedes the earlier open choice between capacity-only
and margin-buffer admission: use a margin buffer for new entries. The percentage
is pending the owner's answer; implementation/test percentages do not activate it.
Retain the current one-position live rule until Strategy 2 has a tested
basket-execution plan. There is no present basket exception or S2 activation.
Routine engineering continues independently. See docs/margin-buffer-entry-policy.md.
