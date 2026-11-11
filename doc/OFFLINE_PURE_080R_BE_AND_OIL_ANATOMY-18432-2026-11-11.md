# Pure +0.80R breakeven and oil-entry anatomy

№18,432 §§1–11 · 11 November 2026 · as of 20:35 SGT · by Codex (runtime model/effort not exposed; Extra High requested).

## TLDR

**Reject this candidate for deployment under the tested assumptions. Removing the +0.50R lock does not remove the historical regression.** The isolated +0.80R → entry stop cuts two historical Tier-1 oil runners and seven of the thirteen actual TP winners in the recent demo ledger.

- Tier 1, 3.74bp: mean net R **1.204630 → 0.809965**; wins **7/7 → 5/7**.
- Tier 1, 24bp stress: **0.419593 → 0.024129R**; the strict-close subset turns slightly negative.
- Tier 2, 3.74bp: unchanged at **0.279986R**, 6/8 forced-close wins.
- Across 35 recent demo trades, paired replay cash falls **$698.03 → $157.77**, a **$540.26 deterioration**. Actual broker cash was **$718.81**; replay estimates are not actual earnings.
- The candidate reduces the selected overnight Crude and 11 November BTC losses, but leaves the 17:14 oil losses unchanged. None of those oil trades came near +0.80R.

All research remained off the trading path. Existing PostgreSQL quotes were copied with explicitly authorised read-only queries; there were **zero broker requests, zero orders, zero production source changes and no deployment**. The baseline policy remains `s3_macro_pullback_owner_18409_v1` on `aebf404d`.

## §1. Scope and evidence boundaries

The plan and its SHA256 were frozen before replay. This run uses exactly the Phase-3/PR214 development manifest and selected cells: 35 Tier-1 sessions at 48bp impulse/3.74bp admission, and 167 Tier-2 sessions at 28bp/3.74bp admission. All 202 native baseline results and normalised input hashes match the frozen results. The 7 and 8 admitted trades are subsets of those denominators, not a new selected list of winners.

“Sealed historical manifest” means frozen inputs here. The protected **19 August–5 September 2025 validation holdout was not opened**. These development dates have already informed earlier hypotheses; this is retrospective comparison, not independent out-of-sample validation. The former 1,212 cells were overlapping admission/threshold variants; only the 202 cases needed for this directive were rerun. Stage 1’s previous zero-entry result was not rerun or counted as evidence for exits.

The live comparison covers 35 post-PR210 demo positions opening from 10 November 00:59 SGT through 11 November 17:14 SGT. A read-only tail reconciliation found **zero new demo deals or S3 fill records in [19:45,20:35) SGT** on 11 November, so the ledger did not change after the preceding audit cutoff. All 13 actual TP winners are included. No live-account orders were involved.

## §2. Frozen exit semantics and costs

The only candidate change is **breakeven trigger 1.00R → 0.80R**. The stop moves to the recorded entry price, never +0.50R. Entry, direction, volume, original R, price precision, TP and each saved deadline are identical between arms. Historical strict and forced-boundary paths preserve their original adapter semantics; they are reported separately.

The old server stop/target is checked before a pending amendment. A proposal becomes active only on a strictly later quote where an entry-price stop is still valid. A gap through an active stop fills at the recorded executable quote, not an invented stop price. This is an optimistic **next-quote acknowledgement model**, not measured broker amendment latency. There is no same-quote retroactive protection and no automatic re-entry after a hypothetical early exit.

Bid/ask spread is already present in executable prices. The 3.74bp and 24bp figures are additional per-side reserves, charged as `(entry + exit) × bps / 10,000 / original R`. For the historical stress row, admission remains frozen at 3.74bp; only costs are repriced at 24bp. That is different from applying a new 24bp admission filter. All actual closing fee components in this scoped live ledger are recorded as zero; cash calculations use those receipts directly, not a price-to-cash rounding remainder. **An entry-price stop is not net breakeven**: spread, costs and stop slippage can make it a loss.

## §3. Historical result, strict versus forced

| Tier / mode | Reserve bp/side | Baseline closed / wins | Candidate closed / wins | Baseline mean net R | Candidate mean net R | Candidate PF |
| --- | --- | --- | --- | --- | --- | --- |
| tier1 / floor=48 / admission=3.74 / forced | 3.74 | 7 / 7 | 7 / 5 | 1.204630 | 0.809965 | 22.4636 |
| tier1 / floor=48 / admission=3.74 / forced | 24.0 | 7 / 7 | 7 / 5 | 0.419593 | 0.024129 | 1.0996 |
| tier1 / floor=48 / admission=3.74 / strict | 3.74 | 6 / 6 | 6 / 4 | 1.234595 | 0.774153 | 18.5839 |
| tier1 / floor=48 / admission=3.74 / strict | 24.0 | 6 / 6 | 6 / 4 | 0.441394 | -0.019980 | 0.9293 |
| tier2 / floor=28 / admission=3.74 / forced | 3.74 | 8 / 6 | 8 / 6 | 0.279986 | 0.279986 | 2.1861 |
| tier2 / floor=28 / admission=3.74 / forced | 24.0 | 8 / 0 | 8 / 0 | -0.940631 | -0.940631 | 0.0000 |
| tier2 / floor=28 / admission=3.74 / strict | 3.74 | 4 / 3 | 4 / 3 | 0.550127 | 0.550127 | 2.7196 |
| tier2 / floor=28 / admission=3.74 / strict | 24.0 | 4 / 0 | 4 / 0 | -0.675294 | -0.675294 | 0.0000 |

Tier 1 has one unresolved strict path in each arm; Tier 2 has four. Forced outcomes mark these at the original boundary and must not be mixed with strict target/stop completions. The two harmed Tier-1 trades are **SpotBrent and SpotCrude on 11 April 2025** (`70e3acf0ef22a39ed2354106`, `ce5a75c11b08e52a205e5350`): both baseline targets of about +1.38R become gross entry-price exits and negative net outcomes after costs. They belong to one correlated oil date, not two independent macro regimes. Tier 2 is byte-for-byte unchanged in exit outcomes.

The earlier bundled lock produced Tier-1 0.595017R; the pure rule is less damaging at 0.809965R, but still worse than 1.204630R. Relative improvement over a rejected candidate is not qualification against the baseline.

## §4. Recent live-ledger counterfactual

| Account | Trades | Actual broker USD | Baseline replay USD | Pure 0.80R replay USD | Actual TP winners cut |
| --- | --- | --- | --- | --- | --- |
| 46130058 | 12 | 455.79 | 450.57 | 69.72 | 3 |
| 47790949 | 12 | 253.24 | 238.39 | 85.69 | 2 |
| 49236051 | 11 | 9.78 | 9.07 | 2.36 | 2 |

| Additional bp/side | Baseline W/L/flat | Candidate W/L/flat | Baseline mean R / PF | Candidate mean R / PF | Baseline / candidate closed-curve max DD R |
| --- | --- | --- | --- | --- | --- |
| 0.0 | 16/19/0 | 6/27/2 | 0.184171 / 1.5191 | 0.049692 / 1.2658 | 9.2588 / 6.2336 |
| 3.74 | 16/19/0 | 6/29/0 | 0.070523 / 1.1675 | -0.064006 / 0.7699 | 10.5711 / 7.8731 |
| 24.0 | 13/22/0 | 6/29/0 | -0.545120 / 0.3220 | -0.679922 / 0.1199 | 19.0792 / 23.7973 |

There is a trade-off: the earlier stop reduces some large losses and the low-cost closed-trade R drawdown, but sacrifices more winner cash and net expectancy. This drawdown is an ordered closed-trade curve, not floating portfolio drawdown. Three accounts often copy one market episode; pooled wins and dollar totals are not independent trials or a replacement for each account’s official latest-20. No simulated trade was appended to live performance.

## §5. Check of all thirteen actual TP winners

The following seven have an observed return to entry after +0.80R and are cut by the candidate. The remaining six retained their target in the paired replay. Times are SGT.

| Position / account | Symbol | First +0.80R | First return to entry | Candidate exit R | Actual TP preserved? |
| --- | --- | --- | --- | --- | --- |
| 260838301 / 49236051 | ETHUSD | 10 Nov 11:25:48.511 | Not observed | 1.380046 | Yes |
| 260838304 / 47790949 | ETHUSD | 10 Nov 11:25:48.511 | Not observed | 1.380046 | Yes |
| 260838310 / 46130058 | ETHUSD | 10 Nov 10:41:18.088 | 10 Nov 10:55:08.277 | -0.021325 | No |
| 260862431 / 49236051 | SpotCrude | 10 Nov 11:46:00.973 | 10 Nov 12:02:07.216 | -0.037383 | No |
| 260862442 / 47790949 | SpotCrude | 10 Nov 11:46:01.172 | 10 Nov 12:02:06.988 | -0.024299 | No |
| 260862445 / 46130058 | SpotCrude | 10 Nov 11:46:01.189 | 10 Nov 12:02:07.014 | -0.024299 | No |
| 260971498 / 46130058 | SpotCrude | 10 Nov 16:30:11.429 | Not observed | 1.380435 | Yes |
| 260982208 / 46130058 | SpotBrent | 10 Nov 17:34:05.879 | Not observed | 1.380769 | Yes |
| 260982991 / 47790949 | SpotCrude | 10 Nov 17:29:49.465 | Not observed | 1.380846 | Yes |
| 260982989 / 49236051 | SpotCrude | 10 Nov 17:29:50.755 | Not observed | 1.381057 | Yes |
| 261128944 / 49236051 | ETHUSD | 10 Nov 21:30:28.534 | 10 Nov 21:30:33.745 | -0.010057 | No |
| 261128943 / 47790949 | ETHUSD | 10 Nov 21:30:28.553 | 10 Nov 21:30:33.766 | -0.010057 | No |
| 261128942 / 46130058 | ETHUSD | 10 Nov 21:30:28.576 | 10 Nov 21:30:33.752 | -0.010057 | No |

The harmed winners are one of the three 10 November morning ETH copies, all three Asian Crude copies, and all three New York ETH copies. The New York ETH return occurred about **5.2 seconds after +0.80R**; exact broker amendment timing is especially material there. The observed return is a quote-path fact; the hypothetical exit requires the specified acknowledgement assumption. Earlier threshold crossing does not guarantee a filled protective stop.

Actual winner cash totals $1,888.84; the baseline replay estimates $1,862.76 and the candidate $817.05. Differences between broker cash and baseline quote replay are shown separately rather than crediting them to the candidate.

## §6. All 35 positions and selected-loss benefit

| Position / account | Symbol | Actual USD | Baseline est. USD | Candidate est. USD | Baseline → candidate gross R |
| --- | --- | --- | --- | --- | --- |
| 260655209 / 46130058 | ETHUSD | 44.97 | 44.89 | -0.26 | 0.3040 → -0.0017 |
| 260655210 / 49236051 | ETHUSD | 1.43 | 1.43 | 0.00 | 0.3057 → -0.0000 |
| 260655217 / 47790949 | ETHUSD | 44.65 | 44.56 | 0.00 | 0.3057 → -0.0000 |
| 260741238 / 47790949 | SpotCrude | -143.72 | -145.43 | -0.86 | -1.0000 → -0.0059 |
| 260741239 / 46130058 | SpotCrude | -150.38 | -147.73 | -11.03 | -1.0030 → -0.0749 |
| 260741495 / 49236051 | SpotCrude | -3.90 | -4.03 | -0.01 | -1.0050 → -0.0030 |
| 260838301 / 49236051 | ETHUSD | 6.58 | 6.52 | 6.52 | 1.3800 → 1.3800 |
| 260838304 / 47790949 | ETHUSD | 201.00 | 200.23 | 200.23 | 1.3800 → 1.3800 |
| 260838310 / 46130058 | ETHUSD | 205.30 | 202.94 | -3.14 | 1.3800 → -0.0213 |
| 260862431 / 49236051 | SpotCrude | 6.11 | 5.91 | -0.16 | 1.3813 → -0.0374 |
| 260862442 / 47790949 | SpotCrude | 201.62 | 200.27 | -3.52 | 1.3813 → -0.0243 |
| 260862445 / 46130058 | SpotCrude | 203.86 | 202.49 | -3.56 | 1.3813 → -0.0243 |
| 260963913 / 49236051 | SpotBrent | -0.02 | -0.02 | -0.02 | -0.0043 → -0.0043 |
| 260963925 / 47790949 | SpotBrent | 0.63 | -0.95 | -0.95 | -0.0065 → -0.0065 |
| 260971498 / 46130058 | SpotCrude | 208.28 | 206.25 | 206.25 | 1.3804 → 1.3804 |
| 260982208 / 46130058 | SpotBrent | 208.44 | 207.86 | 207.86 | 1.3808 → 1.3808 |
| 260982991 / 47790949 | SpotCrude | 216.15 | 203.36 | 203.36 | 1.3808 → 1.3808 |
| 260982989 / 49236051 | SpotCrude | 6.59 | 6.27 | 6.27 | 1.3811 → 1.3811 |
| 260983374 / 47790949 | ETHUSD | -2.82 | -2.82 | -5.26 | -0.0191 → -0.0356 |
| 260983375 / 46130058 | ETHUSD | -3.55 | -2.88 | -5.37 | -0.0191 → -0.0356 |
| 260983373 / 49236051 | ETHUSD | -0.11 | -0.09 | -0.17 | -0.0191 → -0.0356 |
| 261063163 / 47790949 | BTCUSD | -6.68 | -6.12 | -6.12 | -0.0412 → -0.0412 |
| 261063162 / 46130058 | BTCUSD | -6.68 | -6.12 | -6.12 | -0.0409 → -0.0409 |
| 261128944 / 49236051 | ETHUSD | 6.49 | 6.42 | -0.05 | 1.3801 → -0.0101 |
| 261128943 / 47790949 | ETHUSD | 207.05 | 204.98 | -1.49 | 1.3801 → -0.0101 |
| 261128942 / 46130058 | ETHUSD | 211.37 | 209.26 | -1.52 | 1.3801 → -0.0101 |
| 261474373 / 47790949 | ETHUSD | -157.59 | -150.52 | -150.52 | -1.0046 → -1.0046 |
| 261474372 / 49236051 | ETHUSD | -5.02 | -4.93 | -4.93 | -1.0130 → -1.0130 |
| 261474378 / 46130058 | ETHUSD | -161.99 | -155.05 | -155.05 | -1.0130 → -1.0130 |
| 261629338 / 47790949 | BTCUSD | -159.67 | -160.33 | -0.34 | -1.0951 → -0.0023 |
| 261629441 / 49236051 | BTCUSD | -3.22 | -3.38 | -0.06 | -1.0398 → -0.0175 |
| 261629442 / 46130058 | BTCUSD | -148.11 | -155.62 | -2.62 | -1.0398 → -0.0175 |
| 261636289 / 46130058 | SpotCrude | -155.72 | -155.72 | -155.72 | -1.0223 → -1.0223 |
| 261636412 / 49236051 | SpotBrent | -5.15 | -5.03 | -5.03 | -1.0311 → -1.0311 |
| 261636417 / 47790949 | SpotBrent | -147.38 | -148.84 | -148.84 | -1.0000 → -1.0000 |

| Selected cohort | Actual USD | Baseline replay USD | Candidate replay USD |
| --- | --- | --- | --- |
| overnight_crude | -298.00 | -297.19 | -11.90 |
| nov11_btc | -311.00 | -319.33 | -3.02 |
| nov11_oil | -308.25 | -309.59 | -309.59 |
| actual_13_tp_winners | 1888.84 | 1862.76 | 817.05 |

The overnight Crude and 11 November BTC examples do improve, but selecting only those losers would hide the seven sacrificed TP winners, lost overnight ETH time-exit gains and slightly worse slippage on other early exits. The 17:14 oil paths are unchanged because their best observed favourable returns were at most about 0.069R.

## §7. Oil event-local anatomy with parameters frozen

The three loss positions entered at 17:14 SGT on 11 November. The named comparator **260862445 entered at 09:48 SGT in the Asian session and closed at 14:43**, before London; it is retained as requested but must not be labelled a London entry. The other three named comparators are London entries on 10 November.

| Position / symbol | Session open SGT | Entry SGT | Hold seconds | Impulse bp | Touch→turn seconds | Turn fraction | Peer signed bp at guard | Best observed R |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 261636289 / SpotCrude | 11 Nov 15:00:00.000 | 11 Nov 17:14:20.985 | 138.761 | 66.475 | 2.017 | 0.0867 | 54.229 | -0.0848 |
| 261636412 / SpotBrent | 11 Nov 15:00:00.000 | 11 Nov 17:14:37.517 | 206.656 | 85.924 | 12.300 | 0.0821 | 25.541 | 0.0689 |
| 261636417 / SpotBrent | 11 Nov 15:00:00.000 | 11 Nov 17:14:38.559 | 201.725 | 85.924 | 13.312 | 0.0882 | 26.694 | 0.0443 |
| 260862445 / SpotCrude | 10 Nov 07:00:00.000 | 10 Nov 09:48:00.757 | 17730.285 | 82.045 | 14.390 | 0.1011 | 71.297 | 1.3907 |
| 260971498 / SpotCrude | 10 Nov 15:00:00.000 | 10 Nov 16:14:51.447 | 1080.086 | 50.557 | 3.321 | 0.0806 | 42.636 | 1.3940 |
| 260982208 / SpotBrent | 10 Nov 15:00:00.000 | 10 Nov 17:00:20.884 | 2725.996 | 100.052 | 2.609 | 0.0956 | 52.421 | 1.3846 |
| 260982991 / SpotCrude | 10 Nov 15:00:00.000 | 10 Nov 17:01:38.186 | 1969.762 | 67.564 | 4.212 | 0.0862 | 52.099 | 1.4076 |

All seven retained guards pass the frozen 8% turn and 24bp peer requirements, with a touch/turn extreme inside 50%–61.8% retracement. The confirmation quote can already have moved out of that touch band in the intended trade direction; that is the turn mechanic, not proof of an invalid touch.

| Loss position | Opening origin / saved extreme | Entry / SL | First peer <24bp after entry | First opening-origin reclaim | Actual stop fill SGT |
| --- | --- | --- | --- | --- | --- |
| 261636289 | 86.7245 / 86.148 | 86.383 / 86.831 | 11 Nov 17:15:43.924 | 11 Nov 17:15:50.363 | 11 Nov 17:16:39.746 |
| 261636412 | 94.96799999999999 / 94.152 | 94.546 / 95.156 | 11 Nov 17:15:21.194 | 11 Nov 17:16:41.068 | 11 Nov 17:18:04.173 |
| 261636417 | 94.96799999999999 / 94.152 | 94.531 / 95.141 | 11 Nov 17:15:21.196 | 11 Nov 17:16:41.104 | 11 Nov 17:18:00.284 |

**Sequence:** the opening reference and negative multi-day macro context supported a short impulse. Crude armed around 17:01, touched the retracement near 17:14:18 and confirmed after about 2.0 seconds. It sold at 86.383; the peer still showed 54.23bp down from its opening reference. Brent sold roughly 17 seconds later on the other accounts, with only 25.54–26.69bp of remaining peer displacement. Both markets then rallied. Peer displacement fell below 24bp approximately 43–83 seconds after entry; each instrument reclaimed its opening origin before its initial stop was hit. Full event quotes and 1-minute traces from 15:00–17:20 across all three account feeds are in JSON.

The HTML copy includes an annotated price-path chart using one account’s recorded feed. Minute aggregation is for presentation only; the exit replay uses ordered quotes.

## §8. What the anatomy supports—and what it does not

**Supported:** a peer can remain 24bp below its session open while already moving upward against a short. The guard measures displacement, not persistence of the immediate trend. A one-off 8% turn can be a brief fluctuation within a larger rebound. The November 11 guards were valid at dispatch; their successful evaluation did not establish positive expectancy.

**Important counterexamples:** winning London Crude 260971498 confirmed in about 3.3 seconds; winning Brent 260982208 in about 2.6 seconds. Some winners also had positive 1-minute/5-minute price changes before their short entry, or later fell below the peer threshold. The Asian Crude winner reclaimed its opening origin and lost peer displacement before eventually reaching TP. Therefore “wait longer”, “reject any upward pullback”, “exit whenever peer falls below 24bp”, or “close on origin reclaim” are not validated fixes from this small sample.

**Not established:** an external news catalyst, a coding failure in the saved entry guards, or a profitable replacement entry rule. No news/time-of-sales evidence was introduced into this strictly bounded study. The near-term reversal and macro filter disagreement with the later price path are observed; their economic cause is not proven by these quotes alone.

## §9. Verification and limitations

1. Eight focused synthetic semantic cases checked both long and short paths: no trigger below 0.80R; entry-price stop only after a later valid quote; preservation of a runner that retraces below +0.50R; and no retroactive rescue through the original stop. These fixtures are not research trades.
2. All 202 requested historical cases matched frozen native baseline outputs and normalised input hashes. Every paired historical baseline also matched PR214, with unchanged entry and deadline. The frozen plan hash was rechecked.
3. All 35 unique positions reconcile to stored deal receipts. Exactly 13 actual TP winners were checked; seven candidate cuts were identified. Every applied candidate stop equals entry, never +0.50R. Reversing equal-timestamp update order changed zero scoped live outcomes. Seven oil guard checks pass the original geometry.

Recorded quote gaps reach **31.3 seconds** in the Asian oil paths and about **14.1 seconds** in the recent BTC paths. This is not a complete exchange tape. Quote-replay fill price/time is not byte-identical to broker execution: the largest price difference is 16.33 BTC price units, and replay close timing differs by roughly −953 to +23ms. Actual broker results remain authoritative; paired estimates compare model with model.

The next-quote model omits a measured future amend/ack latency distribution, queue/transport failure and changed liquidity from hypothetical orders. It is unsuitable as a live profit guarantee. No causal entry/re-entry feedback simulation was performed: later recorded entries remain fixed even if an earlier hypothetical exit might have altered an account’s subsequent state. The comparison isolates exits, not an entire alternate trading history.

Historical development reuse, cross-account correlation, small admitted samples and unverified round-trip reserves prevent an edge claim. Gross entry stops may still incur negative net cash. Strict and forced outcomes are kept separate; no unseen protected date is consumed.

## §10. Decision and next step

Keep the current production exit unchanged. The isolated candidate fails the comparison on Tier 1 and on the full recent demo ledger, even though it helps the highlighted losers and reduces some low-cost drawdown. It does not qualify the 73% / latest-20 goal or establish a 50% interim class edge.

The useful next investigation is entry-context discrimination: whether multi-day drift plus opening displacement reliably distinguishes continuation from reversal **before** entry. First define an economically motivated, causal hypothesis and its evidence requirements; then evaluate it against both the winning and losing episodes, freeze it, and use separate dates. Do not select a longer turn duration or a new peer threshold from these seven oil paths. If execution-latency modelling is required, use recorded amendment receipts rather than assuming instantaneous protection.

This report authorises no production change. No parameter tuning loop, broker collection, protected-holdout test, Railway deployment or Telegram message was performed.

## §11. Reproduction, files and publication

Plan SHA256: `fdedf21a8caf5bab8d4e6ff38f48b4e7fa46b6882f6e0d5b9ffb6b2577361854`. Native binary SHA256: `3346fc1d7f336285249f8d66b39021e48a152cf939897f8667e284de4a514de3`. The JSON contains the frozen plan, all historical/live paired exits, the 13-winner check, oil event/guard records, minute traces, receipt hashes, private-engine source and offline evaluator/test scripts. The native research copy was built from `8c9bcb9` with only its private candidate proposal changed to entry at 0.80R; it is outside the repository worktree and has no live transport. Current production Rust files were never edited.

To reproduce, use the same repository readers and verified private cache, build the isolated native example offline, then run the embedded historical/live evaluators. Raw market archives are not published; the recorded SHA256 receipts identify the required bytes. Read this Markdown before inspecting the [JSON](OFFLINE_PURE_080R_BE_AND_OIL_ANATOMY-18432-2026-11-11.json). The [HTML](OFFLINE_PURE_080R_BE_AND_OIL_ANATOMY-18432-2026-11-11.html) is a standalone readable copy. Only these three `doc/` files are in the PR. Its final merge receipt and unchanged deployment check will be recorded on the PR after all required CI checks pass.
