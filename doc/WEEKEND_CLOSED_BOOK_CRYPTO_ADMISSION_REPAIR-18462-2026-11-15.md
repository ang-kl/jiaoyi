# №18,462 — Weekend closed-book crypto admission repair

## §1. Outcome and scope

The shared margin gate previously required a live FX conversion quote for every existing foreign position. Consequently a healthy USD crypto candidate could be refused solely because an existing S1 AUS200/GER40 position needed AUD/USD or EUR/USD while FX was closed. This release separates existing-position accounting from candidate and pending-entry valuation. Existing stop liabilities may use broker-calendar-verified pre-close evidence with a 5% adverse FX reserve; candidate quotes and conversion legs remain live and executable.

This is an execution/accounting repair, not strategy qualification or a change in entry/exit rules. PR #230 is merged and deployed; §9 records measured accounting and ownership verification. The restart verification also found a peer-context restoration defect, addressed by the follow-up in this report. Its deployment attestation will be published on the follow-up PR. Future 12:00 SGT opening prices are not claimed before that time.

## §2. Recorded refusals and comparison fills

The read-only PostgreSQL audit covers **14 November 05:35 through 15 November 08:00 SGT, end exclusive**. Account 49236051 recorded the three specified S3 fills: ETHUSD at 14 November 10:41:08 SGT, BTCUSD at 15 November 03:25:53 and 04:21:31 SGT; S1 ETHUSD followed at 06:37:45 SGT. The other two demos recorded no opening fills in that interval.

Both 46130058 and 47790949 recorded `conversion quote stale/incomplete for broker symbol 5` (AUDUSD) during S1 margin admission at **15 November 06:36:45 SGT**, followed by corresponding refused receipts. These are two actual refusals with two diagnostic rows each, not four trading attempts. Live 43002148's contemporaneous S1 refusal was its existing open-risk budget, which remains enforced.

No persisted S3 terminal margin refusal was found at the three S3 comparison entry times on the affected demos. Background preparation failures were not always persisted as terminal receipts. The audit therefore confirms the shared FX failure path but does **not** claim it explains every missed S3 trade. Additional health fields expose known S3 stop risk and commission evidence age/error to separate these issues after release.

## §3. Existing Gold stop liability is already deterministic

The prior `quote_to_deposit` implementation already returned 1 for USD/USD before inspecting market prices. Missing Gold ticks therefore did not, by themselves, prove a broken USD conversion path. This release makes that existing-book identity explicit and tests it with an empty quote cache; authority recovery, an absent stop, or incomplete asset metadata can still correctly refuse.

| Account | Position | Short entry | Initial/current stop | Volume (cents) | Nominal stop loss from entry |
|---|---|---:|---:|---:|---:|
| 46130058 | 262583980 | 3878.12 | 3893.62 | 1000 | $155.00 |
| 47790949 | 262583136 | 3877.19 | 3892.65 | 900 | $139.14 |

The account-wide margin reserve continues to account for broker-reported gross floating P&L relative to the stop. The table is the S3 loss from entry, not a replacement for the full margin calculation. No Gold stop, target, volume, or recorded exit deadline is changed.

## §4. Closed-market FX evidence and boundaries

1. The actual account's broker symbol contract supplies timezone, weekly intervals and holidays. Staleness alone never proves that a market is closed; missing, invalid, disabled or ambiguous calendar evidence refuses.
2. Only **existing open positions** enter the closed-book bucket. Candidate and pending foreign exposures retain live 5-second conversion requirements. The candidate conversion market must also be open even if a subscription response has a recent arrival time.
3. The most recent two-sided pre-close evidence must have both original side timestamps strictly before the current closure and within five seconds of its start. Account-specific broker-dated memory is preferred; otherwise the bounded PostgreSQL tick query reconstructs both sides from one recording session. Historical data never enters the executable quote cache.
4. Evidence is bound to connection generation, account and invalidation revision. Calendar/evidence verification expires after five minutes and is refreshed in the background. Reopening immediately disables historical fallback and reinstates live freshness.
5. For a conversion interval `[low, high]`, apply `low / 1.05` and `high * 1.05` once to the closed conversion chain. Losses use high, profits use low. Missing stops or pre-close evidence refuse; there is no unit-rate substitute for foreign currency.
6. The final guarded writer rechecks these conditions. `crates/jiaoyi-ctrader/src/trading.rs` is byte-identical to the pre-change file.

Stored Friday pre-close evidence was found for AUDUSD and USDJPY across all four accounts. **EURUSD was absent from the bounded Friday tick query.** A broker-dated last quote may be available from the existing subscription on restart; it must satisfy the same provenance checks. If it does not, GER40 exposure correctly remains an admission blocker. This report does not fill that gap with a guessed rate or another account's spread.

## §5. Preserved risk and ownership rules

Strategy policy remains `s3_macro_pullback_owner_18409_v1`. The 0.6% demo trade sizing, 3% S3 open-risk cap, 250% projected margin floor, 1.00R breakeven, 1.38R target, pair reservation and all Three Human Protection Laws remain unchanged. The existing stressed-margin calculation is also unchanged: it requires the stressed level to exceed the broker's recorded margin-call/stop-out warning. The code did not previously impose 250% on both metrics, so this release does not misstate or change that policy.

Live minimum-volume sizing and all S1 trading rules remain intact. No verification order is sent. This change does not guarantee entry when macro/peer/ownership, risk, fee evidence or margin gates refuse.

The prior MARKET_CLOSED deferral implementation is untouched, including its quote-triggered reopen path and retry limits. Regression coverage retains carried Gold deadlines and verifies human/S1 isolation.

## §6. Verification

- `cargo check --workspace --all-targets`: PASS (14.04 seconds).
- `cargo test --workspace`: PASS (322.99 seconds).
- `cargo clippy --workspace -- -D warnings`: PASS (13.43 seconds).
- `cargo fmt --all --check`: PASS (1.13 seconds).
- `cargo test -p jiaoyi --lib cold_start_hydrates_only_existing_book`: PASS (0.94 seconds).
- `cargo test -p jiaoyi-ctrader market_hours::tests`: PASS (10.06 seconds).
- `cargo test -p jiaoyi-ctrader broker_dated_accounting_quotes`: PASS (0.14 seconds).
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS (18.29 seconds).

Workspace test result: **810 passed**, **1 ignored**. Focused cases cover cold-start PostgreSQL hydration without executable quotes; $155 and $139.14 Gold risk; direct/inverse and profit-side adverse reserves; wrong-account/missing/old evidence; weekday 5,001ms rejection; closed-FX candidate rejection; reopen freshness; original broker timestamps and invalidation; no-stop refusal; projected/stressed margin failure; existing restart and MARKET_CLOSED protections.

The initial test environment had a stopped local development PostgreSQL container. Once started, an existing Sunday-only fixture error appeared: the restart test used Friday but only created Saturday-onward tick partitions. Its fixture now creates the actual selected session date. The S1 conversion test also now supplies the open FX contract required by candidate admission, rather than relying on missing calendar metadata. Production session definitions are unchanged.

## §7. Release verification required

CI must pass `check`, `docker`, and `engineering` on the exact PR head before squash merge. After Railway deploys the merge commit, verify four connected/unpaused account loops, the exact 08:00 crypto opening references, known Gold reserves, closed-book quote provenance and per-symbol preparation/refusal diagnostics. Compare live human/S1 inventory and Gold deadlines against the pre-release snapshot; distinguish ordinary autonomous market activity from release interference. Observe the closed-market retry lane over a measured interval, not just a single log line.

12:00 SGT preparation can be inspected in advance; its actual opening quote cannot be certified before that time. Any missing EURUSD evidence or legitimate admission refusal must remain explicit in the release attestation. Financial qualification is not asserted.

## §8. Inspectable evidence

The companion JSON includes exact refusal groups, comparison fill identifiers, Gold geometry, available pre-close quotes, command results, protected-file hash, and SHA256 hashes of the read-only source snapshots. Raw authenticated health and database captures remain private; no tokens or passwords are published. Post-deployment evidence will be linked on the PR so another reviewer can read it on GitHub.

## §9. First rollout verification and required context follow-up

PR #230 passed all three CI jobs and deployed **72276a2624de1f835d1dcc90c857b20332e4b184** on Railway deployment `89de05fc-15b8-481a-ba3f-c3c5b1da5c53`. At 15 November **08:50:18 SGT**, all four accounts were connected and unpaused. The affected demos had authentic broker-dated AUDUSD and EURUSD quotes from **13 November 21:54:59 UTC**, with no closed-book hydration error. EURUSD was absent from the initial Friday tick query but was supplied by the authenticated subscription with its original pre-close timestamp; it was not invented, borrowed across accounts, or relabelled with Sunday's arrival time.

The S3 stop-risk diagnostics showed **$155.00** on 46130058 and **$139.14** on 47790949. Account 46130058 had a currently prepared BTC admission at the second snapshot, demonstrating that the repaired existing-book reserve can pass the actual background admission path. This is preparation, not a fill or a bypass of the final peer/risk/quote guards. Live 43002148 still had a strict USDJPY refusal for pending foreign exposure, which is outside the existing-position exception. Account 49236051's S1 ETHUSD position continued to hold its crypto pair.

All five live human positions, five pending entries and two live S1 positions retained their geometry. Both Gold opening records, stops, targets, quantities and original deadlines were identical. The measured **118.733-second** interval starting 08:48:00 SGT contained zero new session-end attempts and zero MARKET_CLOSED rejections. Engineering deployment and risk settings were unchanged.

### The restart check found a separate context defect

Before deployment, six eligible cells had current 08:00 references. The two displayed references on 49236051 were older 04:00 references under its S1 ownership/pair locks; they were never eight current, entry-ready cells.

After restart, ETH's durable invalidation and the already-evaluated decision mark suppressed the path that restored its current opening reference. The actor restored a current `Day` only when an arm still existed. BTC consequently lacked its sibling's origin despite the original ETH quote remaining in PostgreSQL. This is a cache-restoration defect, not an economic peer disagreement, and it prevented sign-off of full restart readiness.

The follow-up moves current-session `prepare()` outside the optional-arm branch of `restore_pullback()`. It still runs through the existing human lock and bounded original-quote loader. It does not clear invalidation, consume/reset marks, rearm the trade, substitute a current price for the opening price, or change entry/exit thresholds. The original carried-arm source remains independent of the current peer context.

The existing restart regression now uses the real quote-driven path with no arm and an already-evaluated mark, instead of manually calling preparation after restart. It verifies the original tick midpoint and causal macro context, retained invalidation, no rearm, and no orders/amends/closes. The focused restart and durable-cost-veto regressions pass. An old test assertion expected the current cache to remain empty after a cost veto; it now checks that the veto still prevents orders while context is available to the sibling. A separate missing-preparation fixture explicitly clears its cache. The follow-up check results are in the companion JSON. Its required CI must pass before another deployment; measured verification will be published on that PR.

Follow-up local verification: **cargo check --workspace --all-targets**, **cargo test --workspace**, **cargo clippy --workspace -- -D warnings**, and **cargo fmt --all --check** all pass. The full workspace has **810 passed, 0 failed, 1 existing ignored** test.

## §10. Second rollout: references restored; admission expiry remains measurable

PR #231 passed `check`, `docker`, and `engineering` and deployed **b47cac922725377d4a1191a7d48e2f2ed13a2ded**, deployment `b24e0244-9c8c-4187-ad98-51f1cf69ee09`. At **09:17:29 SGT**, all four accounts were connected and unpaused. All six original active crypto references matched their original prices and timestamps exactly; the actor also loaded the current BTC reference on the S1-pair-locked account. **Seven current references** were available. ETH on 49236051 remained withheld by its S1 ownership lock. That is not eight independently tradable cells.

Live human/S1 geometry and both Gold opening records/deadlines were unchanged. Each Gold position produced one reconnect-time MARKET_CLOSED rejection, then returned to the recorded **Monday 16 November 07:01 SGT** probe. The later **130.864-second** observation from 09:15:00 SGT recorded **zero** session-end attempts and **zero** MARKET_CLOSED rejections. This separates reconnect events from continuous retry spam.

The stale AUD/EUR conversion error cleared on both affected demos, but six snapshots from **09:17:33 through 09:20:17 SGT** did not observe a valid prepared admission: ETH reported `prepared admission expired; background refresh required`, while BTC was preparing or expired. Therefore full entry readiness is **not yet signed off**. The live account's pending JPY exposure still correctly requires fresh FX; account 49236051 remains subject to its S1 pair/ownership guards.

The additional infrastructure change batches independent expected-margin reads **two at a time**, preserving sorted-symbol/buy/sell accumulation, all numerical checks and the final book/balance comparison. It does not extend the five-second admission lifetime or quote freshness. A semaphore-controlled broker regression proves both concurrent progress and the two-request bound, with exact projected margin and stop-reserve parity and zero orders/amends/closes. Health now reports each last completed preparation's source age, duration, success/failure and completion time, so stale-source and slow-read explanations can be assessed separately. This is a measured-expiry follow-up, not a claim that transport batching alone has already solved it. Full checks and production verification remain required for this follow-up.

## §11. Timing identified the remaining history/publication defect

PR #232 passed all CI jobs and deployed **08c8eb09b55de77dbee2ce6d3184814dacc2a4b6**. Its new timings made the remaining failure attributable: at the **09:37:30 SGT** snapshot, BTC had successfully prepared in **3.365s** on 46130058 and **3.159s** on 47790949, with original source ages **76ms** and **162ms**. ETH took **7.905s** and **7.641s** and expired. The worker used `join!` and published neither result until both finished, so a successful BTC preparation could expire before publication.

A read-only PostgreSQL audit at **09:39:30 SGT** found:

| Account | Symbol | Opening-attempt keys | Distinct filled positions |
|---|---|---:|---:|
| 46130058 | BTCUSD | 18 | 6 |
| 46130058 | ETHUSD | 20 | 11 |
| 47790949 | BTCUSD | 17 | 7 |
| 47790949 | ETHUSD | 20 | 12 |

The existing Kelly shortcut checked actual position count only below 20 attempt keys. Rejected attempts brought ETH to 20 keys, triggering the full account history read even though 11/12 filled positions cannot contain 20 closed positions. This is unnecessary accounting work before a fresh admission, not a reason to disable Kelly or change its threshold.

The follow-up always applies the existing distinct-position upper-bound query. Below 20 positions, Kelly remains non-operative exactly as before; at 20 or more positions the full history calculation remains mandatory, including when one entry key identifies multiple positions. Each background result is published immediately on completion, with at most the existing two jobs active. The final writer still rejects stale or changed proofs.

Focused regressions pass: 20 attempt keys with 11 distinct positions cannot force the expensive history read (the full-history table is deliberately unavailable in the fixture); duplicate receipts do not inflate the count; one key with 20 positions still invokes full history. A held sibling future also cannot delay publication of the completed result. Required full checks and measured deployment verification remain pending for this follow-up.
