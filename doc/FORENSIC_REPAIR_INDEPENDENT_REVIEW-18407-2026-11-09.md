# Independent implementation review — №18,407

№18,407 §§1–7 · 2026-11-09T17:49:10+08:00 SGT · by Codex (model/effort not exposed to this reviewer).

## §1. Scope and verdict

Read-only independent review of the five owner-authorised production repairs, with particular attention to causal state, account-local peer reservations, restart recovery, final socket guards, existing-position exits and the recorded-path counterfactual. Reviewed working-tree changes based on `9a5b7a3ec251c9aa368b3cab717f80c0660d9456`. The reviewer did not modify production code or commit changes.

No unresolved production correctness defect was identified in the reviewed implementation after the corrections below. This is an implementation review, not a declaration that CI passed, deployment succeeded, all possible defects are absent, or the strategy has a trading edge. Merge/release still requires the parent agent's final build, required CI and deployment verification.

## §2. Production boundaries reviewed

| Boundary | Reviewed behaviour |
|---|---|
| Running impulse | Fresh extensions update the extreme, midpoint and range, reset prior touch evidence, and invalidate cached preparation. Prepared geometry must match the current arm, including asynchronous results from an older request. |
| Touch and turn | Long/short symmetry; first touch in 50–61.8%; strictly later confirmation; deepest observed touch retained; at least an 8% impulse recovery; final quote and recorded confirmation both in 40–61.8%; >61.8% invalidation is terminal for the arm. |
| Durable state | Extensions/touches are saved; invalidations and consumption persist a tombstone. Restored state requires identity, arithmetic and causal checks. Historical default serialisation remains compatible; old arms are not granted invented confirmation. |
| Pair ownership | Account-local broker positions/orders, owned positions and unresolved local intents reserve the sibling pair. Pair blocking disarms the existing sibling setup rather than allowing unobserved price history to be reused after release. |
| Pair dispatch race | Prepared occupancy comes from the exact broker reconciliation snapshot. Account generation/revision/local-entry epoch bind it to admission. Shared account locks serialise consuming sends; final writer rechecks the account revision. Unknown sends remain reserved across restart until explicit terminal/full-fill evidence. |
| Final writer | Fresh proof, current executable price/grid, funded bound, cost reserve, broker calendar and fresh mandatory peer holding at least 24 signed basis points remain mandatory. The fallback path obtains exactly one matching calendar and does not bypass the final proof. |
| Session | New index entries are limited to 09:30 inclusive–16:00 exclusive New York, with DST-aware conversion and the dispatch deadline capped at the entry cutoff. Oil/metals keep the owner-declared London/NY windows; crypto retains six UTC windows. |
| Risk sizing | The demo minimum cannot exceed the ordinary configured per-trade budget (currently 0.6%). Live minimum sizing and existing account-wide live restrictions remain. |
| Existing positions | Policy 18392 remains recognised as two-stage. Recorded exit deadlines and original risk/target plans are recovered under their own policy. The three compiled human/external-trade protections remain. |

## §3. Findings addressed during review

1. **Stale arm after pair occupancy.** An early pair return could skip causal updates while the sibling was active and revive an obsolete arm after closure. The actor now restores, invalidates prepared admission and durably disarms that arm. The added regression exercises restart, blocked quotes and release requiring a new impulse.
2. **Uncertain intent released merely by elapsed time.** Broker-flat reconciliation is not proof that an uncertain request cannot fill. The actor now retains unresolved symbol/risk reservations across restart; explicit rejection/full-fill evidence settles them. A focused regression exercises the uncertain sibling and terminal rejection release.
3. **Cash-close boundary.** The historical broker session ends after the NY cash close. New entry eligibility and the bounded writer deadline now use the separate 16:00 cash cutoff, preserving existing recorded exits. Summer/winter cutoff tests cover the exact boundary.
4. **Counterfactual execution-window deadline.** The native example now stores the current execution window's close on entry, avoiding the original arm's earlier close when a crypto arm carries into another window.
5. **Counterfactual post-consumption retries.** Only unavailable preparation retains the arm. A subsequent final guard/risk refusal consumes the original simulated arm, matching the actor's lifecycle.
6. **Counterfactual risk and price guards.** Held risk uses current stop distance and releases at breakeven. An invented outright spread-widening veto was replaced with the funded executable grid/bound and actual cost-reserve repricing checks.

## §4. Verification evidence and release boundary

The reviewed focused tests cover symmetric state transitions, duplicate/old quote rejection, malformed/restored confirmation state, final proof and peer freshness, revision-bound broker pair occupancy and release, demo minimum risk, cash-session boundaries, durable sibling disarm and unresolved intent recovery. Existing exit/human-protection regressions remain relevant and must pass with the updated entry fixtures.

During review, older entry fixtures exposed a synthetic actor-clock versus real writer-clock mismatch. The explicit existing event-clock offset now translates all three proof timestamps together into the writer's wall-clock domain, preserving causal gaps; normal production uses zero offset. The final freshness/future-date guards remain mandatory. The parent/test agent is responsible for the final test run and should cite only its final results. The independent reviewer has not claimed CI or deployment completion in this file.

## §5. Counterfactual interpretation

The counterfactual is conditional on the original 15 recorded arms and retained chronological bid/ask receipts. It does not reconstruct an entire alternative trading day, create new arms after disarming, reproduce asynchronous admission readiness or reconstruct counterfactual account equity/margin/manual orders. Broker fills, stop transport and breakeven acknowledgements remain modelled. Its monetary result subtracts the disclosed 3.74 bp/side reserve and recorded commission from executable-quote P&L; recorded bid/ask already accounts for spread.

Publication must retain the following limits:

- The revised rules do not avoid every losing trade: the current recorded-path result retains a SpotCrude stop loss; ETH qualifies and reaches its modelled target; Brent is pair-blocked; the London index setups are excluded by the new session rule.
- Removing the pair guard is a diagnostic attribution arm, not an alternative selected because it performs better.
- Same-time quote ordering and any missing coverage must remain disclosed. Open hypothetical positions produce null total counterfactual P&L until resolved; no convenient historical exit may be fabricated.
- The repairs were selected after observing this day. Improved retrospective P&L is not independent evidence of an edge, a 73% win-rate qualification or a financial PASS.

## §6. Remaining operational limitations

The two-step rule establishes a recovery from the deepest observed touch. It does not prove that every final tick is moving in the impulse direction, or guarantee that the market will continue. Do not describe it as a guarantee to never sell a rising market/buy a falling market.

Account revision and admission locks define the program's atomic execution boundary; they cannot make an external broker/human action atomic with the program's network send. Fresh broker reconciliation, revision invalidation, durable uncertain-intent reservations and exact ownership checks are the practical controls. Uncertain intent reservations may require explicit reconciliation evidence rather than time-based automatic release.

## §7. Required final evidence

Before declaring the directive complete, record the final required CI results, merged source SHA, Railway deployment SHA/status, compiled policy identity and each account loop's connected/unpaused state. Report any live position conservatively under its recorded policy; do not send a discretionary test order merely to prove deployment.
