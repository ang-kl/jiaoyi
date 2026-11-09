# Restore native index sessions and expose entry gates — №18,409

№18,409 §§1–8 · 2026-11-09T19:13:23+08:00 SGT · by Codex (GPT-6), effort not exposed.

## TLDR

The owner authorised reversal of PR #210 Repair 4. Policy `s3_macro_pullback_owner_18409_v1` restores the six traditional core symbols to their native Asian, London and New York entry windows. All other repairs remain enforced. The dashboard now shows the actual macro, peer, pair and setup gates next to numeric distance. A narrowly scoped migration reuses compatible PR #210 preparation from PostgreSQL while retaining its original registration hash and causal timestamps.

This report records implementation and local evidence. Required CI, merge and actual Railway deployment are separate release gates; the PR's release-verification comment supplies the final deployed SHA, account status and before/after opening-reference comparison. A green build alone is not deployment proof.

## §1. Why the change was necessary

The №18,408 investigation found four connected, unpaused account actors with advancing exit checks. Inactivity was explained by macro vetoes, insufficient impulses and the newly imposed NY-only index rule. Numeric distance could be negative while `reason` was null, hiding a macro rejection. Removing the session restriction does not remove the impulse, macro, cost, peer, ownership or risk requirements and does not promise an order.

## §2. Active session policy

| Instrument group | Enabled native windows | Boundaries retained |
|---|---|---|
| US500 / NAS100 | Asian, London, New York | London07:00UTC; NY09:30 America/New_York; broker maintenance/close |
| SpotCrude / SpotBrent / XAUUSD / XAGUSD | Asian, London, New York | Same slot calendar and each instrument's actual broker opening |
| BTCUSD / ETHUSD | Six UTC four-hour windows | Broker maintenance remains excluded |

On 9 November, index windows correspond to07:00–15:00SGT,15:00–22:30SGT,22:30–05:59SGT. New York follows DST. Metals' Asian executable opening is delayed one minute; Brent's Tuesday–Friday Asian opening follows20:00NY; Friday close is16:55NY. This restores all **active broker** windows, not permission during a broker closure. The final broker holiday/trading-mode guard remains mandatory.

Historical18407 forensic replay calls its explicit historical session helper and retains its old policy identity. Previously opened positions retain their recorded exit deadline and recognised two-stage exit policy.

## §3. Repairs retained

| Repair | Retained mechanism | Verification |
|---|---|---|
| 1 | Account-local pair reservation includes active, pending and uncertain sibling exposure; restart tombstones prevent resurrection | Existing actor and final-writer regressions |
| 2 | Running extreme updates reference; deeper than61.8% invalidates | Existing core tests; actor restart/invalidation test plus new visible-state assertions |
| 3 | Fresh50–61.8% touch, later8% turn, confirmation bounds40–61.8%; mandatory fresh24bp peer at dispatch | Core/writer/actor tests; prepared-runtime test; peer display changes with freshness |
| 5 | Demo broker-minimum volume must fit ordinary configured0.6% trade-risk budget | Existing minimum-volume and aggregate-risk tests |
| Human protection | Exact `jiaoyi-s3` ownership/provenance, yield to human position/order, NatGas/JPN225 reserved | Existing ownership, restart and no-amend tests |

The operational3.74bp/side allowance, observed spread and recorded commission remain unchanged. No discretionary verification order or Telegram message is part of this release.

## §4. Health and dashboard contract

Each S3 `trigger_distance` item now includes `entry_gate`, all `entry_gates`, `entry_session_enabled`, `impulse_bps`, macro drift percentages, signed peer movement, `last_assessment`, and `opening_reference` with original quote evidence and preparation context. The first gate is the primary explanation; the UI displays every gate independently of numeric distance. Reasons are escaped text.

Displayed states include macro opposition with5d/20d percentages; missing/stale/weak/opposing peer; sibling pair reservation; armed touch/turn reference and extreme; and deep-retracement invalidation. Peer24bp is explicitly labelled as mandatory at dispatch: an arm may exist while that dispatch gate is unsatisfied. Invalidated setup tombstones restore after an actor restart. This diagnostics path reads actor memory and grants no order authority.

The existing first20rows, paging, scroll region and active-page update behaviour remain intact.

## §5. Restart preservation

Before release, PostgreSQL supplied **32/32** account-symbol opening receipts: six London07:00UTC references and the current crypto08:00UTC references on each of the four accounts. Their exact quotes, timestamps, macro context and source hash are in `docs/production_update_18409_verification.json`.

Preparation had been registration-hash scoped. Only the exact18407 registration is now accepted as a compatible preparation source. The code checks account/symbol/session identity, causal context cutoff, finite drifts/coefficients and preceding session dates. It preserves the source hash rather than relabelling evidence. Unknown registrations and future/noncausal preparation are rejected. Setup arms are still registration scoped; this compatibility does not promote an old arm into a new execution.

Opening midpoints continue to come from the original account-scoped PostgreSQL ticks within the native opening six seconds. The restart test uses deliberately different M30 prices and verifies that neither the M30 opening price nor the restart quote replaces that midpoint. Two actor instances restore all eight symbols; only16current-window M30 finalisation requests occur, with **zero repeated historical preparation downloads**.

## §6. Verification

- PostgreSQL integration: all eight symbols retain original opening midpoint, source timestamp, macro context and sigma table across restart, including the prior registration; no trade/amend/close frames.
- Session tests cover both summer/winter, all three traditional windows, exact closes, forged sessions, crypto maintenance, and unchanged historical18407 restrictions.
- Health tests cover numeric distance with macro veto, peer freshness, pair reservation, durable invalidation and the existing cost blocker.
- Chromium: **11 checks passed**, including all five requested gate states, numeric distance,20/5paging, escaping, mobile scrolling and no JavaScript errors.
- Workspace test totals at report generation: **790 passed, 0 failed, 1 existing ignored**; completion flag `true`. Final CI runs the full required suites.
- Required release gates: `check`, `docker`, `engineering`, then actual Railway source/policy/account/reference verification. No lint or CI gate has been bypassed.

Reproduce browser check by generating HTML with `JIAOYI_UI_FIXTURE_PATH=/tmp/veto-fixture.html cargo test -p jiaoyi --test trigger_veto_ui`, then running `node crates/jiaoyi/tests/web/trigger_veto_browser.cjs /tmp/veto-fixture.html` with Playwright and Chromium installed.

## §7. Goal and limits

The invariant remains73% net wins over the latest20trades, separately for each strategy/account, with the1:1.38 reward/risk objective. This release fixes eligibility, evidence continuity and observability. It does **not** establish a financial PASS, an improved win rate or a new backtest result. An allowed session can legitimately remain idle when another gate rejects the setup.

## §8. Reading order for another AI

Read this report, `docs/production_update_18409_verification.json`, the18409 registration, and the PR's final release-verification comment. Inspect `production_sessions.rs`, `strategy3/diagnostics.rs`, `strategy3/preopen.rs`, and the focused restart/UI tests. Preserve Repairs1,2,3,5 and all three human laws. Do not treat the historical18407 counterfactual as evidence for18409's broader sessions.

## §9. First deployment verified; preparation-refusal visibility follow-up

PR #211 merged at19:25:54SGT. Railway deployment `fa3ee98d-662a-4b71-80c6-f7bebee714be` succeeded on commit `3cbc61d50021892784807c22078507757715a6ca`. At19:28SGT all four accounts were connected and unpaused under18409. All32original opening midpoints, quote timestamps and macro contexts matched the pre-release snapshot; all32current native sessions were enabled. Each account's exit-check counter subsequently advanced by13. The actual dashboard rendered80gate cells and explicit macro/peer vetoes.

The live read revealed another material display omission: SpotCrude and SpotBrent were armed, but their prepared admission was refused by the unchanged margin buffer (approximately209–210%projected margin versus the250%floor). The health payload already held that refusal; the gate column did not. The follow-up displays unavailable admission, original-source refusal or the latest margin/risk refusal alongside the armed setup. A current valid plan suppresses an old refusal. A pending plan with no refusal is labelled preparing, with no invented reason. No strategy, risk, order or session code changes in this follow-up.

A focused regression covers refusal visibility, source-error priority and stale-refusal clearance. The browser fixture additionally exercises the margin message. Final follow-up CI/deployment evidence is appended to its PR release-verification comment.
