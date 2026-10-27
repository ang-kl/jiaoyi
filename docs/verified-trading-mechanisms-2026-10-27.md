# Verified trading mechanisms: trailing-stop evidence

Document ENG-20261027-TRAIL-MECHANISMS. Generated 27 October 2026, 16:14 SGT (Asia/Singapore). Source inspection cutoff: 16:12 SGT. This is an engineering evidence audit, not a change to trading rules or financial acceptance.

## 1. Scope and conclusion

Trailing-stop mechanisms are useful for maintaining protection: advance the stop only in the position's favour, use the executable exit quote, preserve other broker protection, and act only on owned positions. Jiaoyi already implements these mechanisms for registered Strategy 1. Their presence and passing safety tests do not establish that a different trailing policy improves net returns. Strategy 3's registration explicitly says **no stop amends** (§17); its original stop, TP1/TP2 and session exit remain authoritative.

The bot-trade reference is pinned to [`2be966ebaf9a3d6efc533b480d0cd4e7b33f6547`](https://github.com/ang-kl/bot-trade/tree/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547). Existing comparison files and read-only GitHub contents at that exact revision were inspected. Jiaoyi source was inspected at local HEAD `f5f21dd7831cb81a916481ec33d5f4a7e8e1e0a9`. No production endpoint, broker, deployment control or live settings were accessed. Existing R1/R2 work stays paused.

## 2. Source → check → status

“Passed” below means an executed local test. “Source checked” means implementation and assertions were inspected; it does not claim those tests ran. Native C++/Node tests were not executed in this bounded audit.

| Mechanism / reusable invariant | Pinned bot-trade source and relevant check | Existing jiaoyi source and actual check | Status / limit |
|---|---|---|---|
| Never loosen a known stop; compare with fresh broker protection | `cpp-exec/src/trail_engine.cpp:24–49`; `test_trail_engine.cpp` tests `testLongRatchet`, `testShortRatchet`, `testUnknownSlNeverAmends`. `test_protection_ratchet.cpp:95–118,143–165` checks stale targets, concurrent ratchets and failed pre-read. | `crates/jiaoyi/src/trading.rs::trail` requires strict improvement. Gate `check` rejects worsening cached stops; `send` rechecks the fresh broker stop. Tests `the_trail_only_moves_in_the_trades_favour`, `an_amend_moves_an_owned_stop_only_in_the_trades_favour`, and `an_amend_carries_the_take_profit_and_passes_the_brokers_own_stop`. | **Passed:** pure long/short ratchet and both fake-broker gate checks. Bot-trade assertions source checked. An unknown cached/native stop is not evidence that a tighter broker stop does not exist. |
| Long follows bid; short follows ask | Native `trailDecide` explicitly selects bid for long, ask for short; `testLongRatchet`/`testShortRatchet` exercise those sides. | S1 `on_quote` chooses `sym.bid` for buy positions and `sym.ask` for sell positions. Existing SQLx `a_demo_account_sends_one_entry_at_once_and_trails_it` checks bid 3700 minus 3 ATR = 3580. Existing research test `a_short_stop_trails_on_the_ask_and_never_loosens` checks short ask-basis exits. | **Source checked.** Pure long/short ratchet passed; the selected live-loop SQLx test could not reach local DB setup. Research-engine fixture is useful offline evidence, not a live-loop short restart test. |
| Preserve the current broker TP when amending | Native `buildTrailAmend` requests ratchet-only and required TP; the protection transaction reads current TP instead of trusting the configured snapshot. `directTransactions` checks read-back TP and rejects corrupted post-amend protection. | Gate `send` reconciles before amend, carries finite positive `p.take_profit` into the replacing amend and saved intent. Fake broker also replaces protection. `an_amend_carries_the_take_profit_and_passes_the_brokers_own_stop` asserts TP remains 3500 on frame, fake broker and intent. | **Passed:** jiaoyi fake-broker TP preservation. Native read/amend/read assertions source checked. S1 legitimately has no TP; native `configure` rejects targetless specs, so copying that engine unchanged would disable S1 trailing. |
| Only the program's own positions may be managed | Coordinator checks account credentials/registered side and rejects foreign-account specs or duplicate identities; native builder includes account/symbol/direction. | S1 reconcile requires its `jiaoyi-s1` label plus owned open-intent/fill evidence before populating `book.own`; gate independently calls ledger ownership. `a_position_this_program_did_not_open_is_never_amended_or_closed` asserts refusal. | **Passed:** gate ownership refusal. Source checked for S1 reconcile. Native account/identity safeguards are useful invariants, not permission to adopt external positions. Jiaoyi's fresh amend check does not independently assert the caller's side against broker trade-side; no wrong-side failure was reproduced here. |
| Pace broker work and avoid duplicate pending work | Guardian single-flight/cooldown; coordinator accepted-set digest distinguishes configuration replacement from routine stop/peak refresh. Native `onTick` only computes pending work; a worker sends it. Guardian/coordinator tests cover unchanged-set throttle, changed sets, refused retry and independent account populations. | S1 `on_quote` keeps one `last_amend` per position, checks existing `trading.trail_amend_secs` before settings/gate reads, and sets the timer before the attempt. Intent keys include account/position/new stop. | **Source checked; no dedicated pacing-boundary test found in the inspected S1 tests.** Bot native's 10%-of-distance minimum step, 200ms worker and configuration refresh cadence differ from S1's existing interval. A configuration-push throttle is not an amend-rate proof. No new cadence or threshold is adopted. |
| Recover without surrendering an already tighter stop | Native `configure` retains further-advanced local peak/SL only for matching account/symbol/direction, clears omitted positions and changes generation so stale completions cannot own a new config. `testConfigureKeepsLocalProgress` covers a stale push, not a full process restart. Coordinator tests retain durable clear/retry obligations across restart. | S1 reconcile adopts the broker's stop; new local positions start with `best=None,last_amend=None`. On next quote, best is reconstructed from broker stop ±3×current ATR and executable quote; the gate still refuses loosening. Existing `an_accepted_only_open_is_adopted_after_a_restart` asserts adoption and no resend. | **Source checked.** Exact pre-restart peak recovery after changing ATR, and durable pacing across process restart, are not established by those tests. The broker-stop floor is reusable; exact historical peak continuity must not be claimed. |
| Distinguish an amend response from independently confirmed protection movement | `protection_ratchet.hpp::confirmedMovementProof` requires matching account/position/symbol/direction/entry, ordered broker reads, a strictly improved SL and an actual level amend. `confirmedMovementTransactions`, `confirmedMovementIdentityBoundaries`, `confirmedTrailRingReceipt` reject policy-only/no-op/identity-change claims. | Gate completes an amend from its execution response (`Outcome::Amended`); S1 then updates its cached stop. There is a fresh pre-read but no immediate post-amend reconcile in this path. | **Evidence gap, not a reproduced defect:** an execution acknowledgement is not the native mechanism's independent read-back proof. Do not label cached/requested SL or a policy stamp as a verified broker level movement. |
| Preserve precision and registered strategy boundaries | Native trail uses nearest rounding and requires a minimum target step. `stopLocksProfit` is a price comparison at entry, without trade costs. | S1 `round_stop` floors long stops and ceils short stops; `trail` uses the registered 3 ATR. S1 registration says no TP. S3 registration §17 forbids stop amends. | **Source checked.** Native rounding/step/profit-lock thresholds are policy, not drop-in settings. A stop at entry is not a proven net-of-cost breakeven or net win. |

Pinned native source locations: [trail engine](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/cpp-exec/src/trail_engine.cpp), [pure trail tests](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/cpp-exec/src/tests/test_trail_engine.cpp), [protection transaction tests](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/cpp-exec/src/tests/test_protection_ratchet.cpp), [movement-proof helper](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/cpp-exec/src/protection_ratchet.hpp), [guardian](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/agent/services/guardian.js), [guardian tests](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/agent/services/guardian.test.js), [coordinator](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/agent/services/trail-config-coordinator.js), [coordinator tests](https://github.com/ang-kl/bot-trade/blob/2be966ebaf9a3d6efc533b480d0cd4e7b33f6547/agent/services/trail-config-coordinator.test.js).

## 3. Executed checks and unavailable checks

With `/workspace/.jiaoyi-env/activate.sh` sourced:

```sh
cargo test -p jiaoyi --lib the_trail_only_moves_in_the_trades_favour
cargo test -p jiaoyi-ctrader --test trading_gate amend
```

Results: **1 pure test passed; 3 fake-broker gate tests passed; 0 failed/ignored**. These verify long/short non-loosening, fresh broker-stop refusal, TP preservation and ownership refusal.

Two existing SQLx tests were attempted without changing their source:

```sh
cargo test -p jiaoyi --test trading a_demo_account_sends_one_entry_at_once_and_trails_it -- --exact
cargo test -p jiaoyi --test trading the_owners_pause_stops_new_entries_and_the_trail_still_runs -- --exact
```

Both failed at SQLx setup with `PoolTimedOut` after 30s, before their assertions ran; loopback PostgreSQL port 5432 refused connection. This is **Not Verifiable in that run**, not a demonstrated trailing defect or a behavioral red. The environment was not changed and no repeated attempt or broader suite was made. When a local test DB is available, these exact two tests are the bounded rerun; the existing research short-ask and accepted-open restart tests are useful additional targeted checks only if those particular claims are needed.

## 4. Missing validation and decision boundary

No current jiaoyi trailing failure was reproduced in this audit. The verified safety mechanisms can be reused as offline candidate invariants. Missing direct coverage is specific: pacing boundaries/restart cooldown, original peak continuity when ATR changes across restart, and independently confirmed broker protection movement. Native test source suggests concrete negative cases for those claims; it does not prove the current runtime fails them.

Investment validation remains separate: compare a fixed, registered baseline with a separately recorded offline candidate on identical executable bid/ask paths, original initial-risk denominators and complete broker-supported costs. Include long/short, gaps, spread widening, no-op/rejected amends, and restart/identity cases; retain unsupported execution evidence as unknown. A candidate's superior stop location or in-sample payoff is not proof of improved net returns or repeatable 73% wins/1.38 realized payoff. No counterfactual profit is asserted from incomplete post-stop ticks, no live parameters or strategy registration are changed, and no research job is unpaused by this document.

## 5. Addendum — restored test database, 27 October 2026, 16:32 SGT

The parent restored the isolated local PostgreSQL test database and reran the
same two existing S1 SQLx tests without weakening their assertions. Retain the
earlier `PoolTimedOut`/connection-refused attempts above as setup failures;
they were not behavioral failures and are not erased by these later results.

| Existing test | Actual rerun result | Local receipt |
|---|---|---|
| `a_demo_account_sends_one_entry_at_once_and_trails_it` | 1 passed, 0 failed/ignored; 5.38s | `/workspace/.jiaoyi-env/validation-s1-live-trail-test.log` |
| `the_owners_pause_stops_new_entries_and_the_trail_still_runs` | 1 passed, 0 failed/ignored; 7.37s | `/workspace/.jiaoyi-env/validation-s1-pause-trail-test.log` |

The receipt tails were inspected for this addendum. These are local real-client /
FakeBroker / PostgreSQL integration checks, not live-broker trades. They now
support the tested S1 entry/trail path and preserved exits under an owner pause.
They do not prove short-side restart behavior, durable pacing, exact historical
peak recovery or independently read-back broker protection movement. Those
specific limits above remain. No live S3 trailing policy or research pause was
changed by these tests or this addendum.
