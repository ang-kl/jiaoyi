# Strategy 3 production repair registration — №18,407

№18,407 §§1–5 · 09 November 2026 SGT · by Codex (GPT-6), effort not exposed.

The owner explicitly authorises policy `s3_macro_pullback_owner_18407_v1` for new production entries. This is an experimental economic-policy change prompted by the №18,406 forensic review, not proof of an edge or financial PASS. Model family remains `s3_macro_pullback_v1`. The project goal remains 73% net wins over the latest 20 complete trades, separately per strategy/account, and the 1:1.38 reward/risk objective. Time is money; preparation, source identity and causal evidence are invariants.

## New-entry rules

1. Account-scoped pairs are SpotCrude/SpotBrent, US500/NAS100, XAUUSD/XAGUSD and BTCUSD/ETHUSD. At most one pair leg may have an active position, pending order or consuming/uncertain entry on an account. Broker-book and local durable-intent evidence must prevent sibling dispatch, including concurrent changes and restart. The existing aggregate S3 stop-risk and margin limits remain mandatory. An unresolved or unverifiable reservation cannot grant execution permission.
2. After a qualifying causal opening impulse of at least 48 bp, update the running impulse extreme on every fresh quote extending in its direction. Recompute the 50% reference and invalidate/rebuild prepared admission. Reset prior touch/turn evidence on extension. No future quote may determine an earlier extreme or entry.
3. A retracement greater than 61.8% disarms before dispatch. A fresh 50–61.8% zone touch records the deepest pullback midpoint. A subsequent fresh quote must turn at least 8% of the current full impulse in its original direction while remaining inside the 40–61.8% retracement zone. The touch quote itself cannot confirm. Final writer checks must validate the proof and current bounds; the legacy simple-touch bypass is not valid for this policy.
4. At dispatch a fresh peer must retain at least 24 bp of net movement from its own session opening reference in the same direction. Missing, stale, conflicting or weaker peer evidence refuses entry.
5. US500/NAS100 new entries use only the 09:30 inclusive to 16:00 exclusive America/New_York cash session, DST-aware. Existing position deadlines remain recorded under their own policy. SpotCrude/SpotBrent/XAUUSD/XAGUSD new entries use the owner-declared 07:00 UTC London window and New York window. Crypto retains six UTC four-hour windows. Historical window identities and previously recorded position deadlines do not change.
6. Demo minimum volume is admitted only within the ordinary configured trade-risk budget (currently 0.6% of equity); no discretionary 0.75% rounding exception is used. If the smallest legal size exceeds that budget, refuse with `min_volume_exceeds_trade_risk_budget`. Live minimum-volume and one-program-position protections retain their existing authority.

There is at most one recorded entry per symbol/window. The 50% touch is evidence for the later turn, not immediate dispatch authority. Pair reservations also cover active exposure carried across window boundaries.

## Execution, management and evidence

Keep the operational 3.74 bp/side cost allowance plus observed spread and recorded commission, outward-rounded structural brackets, +1R entry-price breakeven and +1.38R gross target. The allowance is an assumption, not verified universal round-trip cost. Prepared relative brackets fund the adverse dispatch bound; the broker anchors them to its actual fill. Market transport and stop gaps do not guarantee the planned cash loss is the realised maximum.

Preserve the three compiled human protections: exact current `jiaoyi-s3` label plus provenance for management, yield to external/human account-symbol positions and orders, and permanent NatGas/JPN225 reservation. Never rewrite an already-open position's original stop, target, original R or recorded exit deadline during the upgrade. Keep all supported historical two-stage policies, including 18392, recognisable for exits. Old pending setup arms must not be silently upgraded with invented turn evidence.

Today's 15 executions are development evidence. Their path aggregates alone cannot support exact replay; use retained sequential bid/ask receipts and causal peer observations. Counterfactual entries and fills are modelled, not observed broker executions. Report sampling/recording gaps, no-entries, unresolved trades and explicit cost assumptions. Do not force-close an unresolved candidate at the old policy's exit timestamp to produce a convenient result. Unseen-date validation is separate and remains required before an edge claim.

Validate five repair boundaries and preserved human ownership with focused tests; use required check/docker/engineering CI before squash merge. Verify the actual Railway source SHA, policy identity, connected account loops and unpaused state after deployment. No manual verification order is authorised by this registration.
