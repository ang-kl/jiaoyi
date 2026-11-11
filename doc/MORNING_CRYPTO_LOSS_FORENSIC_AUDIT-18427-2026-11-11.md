## TLDR — Morning crypto loss forensic audit №18,427

Three ETH shorts lost $324.60: demo46130058 −$161.99, demo47790949 −$157.59, small demo49236051 −$5.02. Live lost $0 from bot trading in this window. No BTC entry filled; three later attempts were refused by the final peer guard. No S3 position remains open. The evidence supports a failed short-term entry followed by stop slippage, not deployment-hash re-entry or a second losing BTC leg. No fix is implemented.

№18,427 §§1–12 ·11-11-2026 · by Codex (GPT-6), effort not exposed. Activity cutoff 12:55SGT is exclusive. Goal remains 73% net wins on latest 20 per strategy/account and 1:1.38 reference; financial qualification is false. Latest20 remains live 3/20 and each demo 8/20: no win-rate progress since№18,425.

## §1. Scope, cutover and evidence

The directive’s approximate09:03 cutover is corrected by deployment receipts: actor recovery occurred08:55:56–58 and the new release was independently verified running08:56:39. All morning fills occurred09:14, after this cutover. The registration text is byte-identical across ef046993 and aebf404; its SHA256 is0444a4c326c391638f3b2e90c9eb5790e15cbeb42e943dfb7735aeed1287534c. Source: strategy3.rs prereg_hash() and the two Git objects. Only doc/ report files are published; no journal or production files are edited. PR is deliberately left unmerged.

No new broker calls, orders, pause changes or redeployment. Existing PostgreSQL records and owner snapshots only.

History polls extend beyond 04:55 UTC cutoff; reported window metrics are strictly filtered before cutoff. Raw s3_orders and s3_fills retain supporting lookback from 00:00 SGT, including earlier refusals; these are not counted as window activity.

Snapshot balances/float captured12:55:52–53SGT, books around12:56SGT: not atomic12:55 historical marks.

Per-position net unrealized breakdown not present in selected snapshot; account aggregate shown, per-position null.

Tick extrema use retained simultaneous bid/ask records, not a guarantee of every upstream quote; max trade-path gap9.30s.

Current macro preparation labels are technical price drift, not fundamental news evidence.

Deployed touch zone is 50.0–61.8%; confirmation zone is 40.0–61.8% after an 8% turn. Observed confirmation around 41% satisfies code, not a narrower 50–61.8% confirmation interpretation.

## §2. All-account balances, float, realized cash and goals

| Account | Balance USD | Float USD | Equity USD | Margin assessment % | Window S3 net | S1/S2/external realized | Latest20 wins |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 501.84 | 4482.19 | 4984.03 | 1038.8276020269848 | 0.0 | 0 / 0 / 0 | 3/20 |
| 46130058 | 25386.63 | -23.34 | 25363.29 | 50910.03613006825 | -161.99 | 0 / 0 / 0 | 8/20 |
| 47790949 | 24850.64 | -16.68 | 24833.96 | 6234.675637678249 | -157.59 | 0 / 0 / 0 | 8/20 |
| 49236051 | 815.39 | 0.0 | 815.39 | Not Verifiable | -5.02 | 0 / 0 / 0 | 8/20 |

Margin is the separately timestamped owner-assessment measure; the accounting snapshot does not certify broker account aggregation. No-position small demo has no finite margin percentage. Combined demo/live totals would mix real and simulated capital and are intentionally not a portfolio return.

## §3. Every filled/open/closed position and pending human order

| Account | Strategy | Symbol | Position/order ID | State | Side | Native volume | Entry | SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | human_external | JPN225 | 649992251 | position | buy | 4000 | 57126.25 | 56571.3 | Not Verifiable |
| 43002148 | human_external | JPN225 | 649772867 | position | buy | 12000 | 57372.7 | 56450.7 | Not Verifiable |
| 43002148 | human_external | NatGas | 637075128 | position | buy | 150000 | 2.52 | 2.327 | 3.875 |
| 43002148 | human_external | XAUUSD | 400465199 | pending_entry | buy | 1000 | 3960.28 | 3877.3 | 4339.75 |
| 43002148 | human_external | JPN225 | 400937857 | pending_entry | buy | 5000 | 59631.3 | 59366.5 | Not Verifiable |
| 43002148 | human_external | NatGas | 398403207 | pending_entry | buy | 200000 | 2.467 | 2.381 | 4.275 |
| 43002148 | human_external | JPN225 | 400083120 | pending_entry | buy | 15000 | 56232.6 | 55679.5 | Not Verifiable |
| 43002148 | human_external | NatGas | 400920027 | pending_entry | buy | 100000 | 2.581 | 2.423 | Not Verifiable |
| 46130058 | s1 | AUS200 | 260823431 | position | buy | 60 | 9142.6 | 8817.1 | Not Verifiable |
| 46130058 | s1 | AUDUSD | 261231902 | position | sell | 900000 | 0.68521 | 0.69978 | Not Verifiable |
| 47790949 | s1 | AUS200 | 260743517 | position | buy | 60 | 9126.3 | 8817.1 | Not Verifiable |
| 47790949 | s1 | AUDUSD | 261231903 | position | sell | 900000 | 0.68526 | 0.69973 | Not Verifiable |

The four S1 positions are carried AUS200/AUDUSD positions on the two larger demos. No new S1/S2 fill, stop amendment or close occurred inside this window. Two RECONCILED_FILL events at redeployment are rediscovery of existing positions, not new orders. Live holds three human positions and five human pending entries; zero live bot intents/events are recorded in the window. This is observed non-interference, not a complete certificate for every historical human amendment. NatGas currently has stale/unverified quote evidence; account aggregate broker P&L is available but a fresh asset-level mark is not certified.

## §4. Every ETH closed trade

| Account / position | Entry SGT | Entry | SL | TP | Exit SGT | Exit | Net USD | Net R | Native volume |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 261474373 | 2026-11-11T09:14:06.521000+08:00 | 2042.83 | 2053.64 | 2027.91 | 2026-11-11T09:29:25.471000+08:00 | 2054.2 | -157.59 | -1.051816 | 1386 |
| 49236051 / 261474372 | 2026-11-11T09:14:06.403000+08:00 | 2042.98 | 2053.79 | 2028.06 | 2026-11-11T09:29:25.678000+08:00 | 2054.14 | -5.02 | -1.031966 | 45 |
| 46130058 / 261474378 | 2026-11-11T09:14:07.305000+08:00 | 2042.98 | 2053.79 | 2028.06 | 2026-11-11T09:29:25.646000+08:00 | 2054.42 | -161.99 | -1.058277 | 1416 |

All three are jiaoyi-s3 shorts in 08:00–12:00SGT with arms formed 09:08:41. No carried arm is involved. All protective exits are original stop-loss fills; no breakeven amendment occurred. Actual opening and closing broker deal volumes reconcile exactly.

## §5. Signal, touch, turn and final peer anatomy

| Account | Origin | Initial→running extreme | Pullback ref | Touch SGT | Touch retrace % | Confirm SGT | Confirm retrace % | Turn fraction | Peer arm/final signed bp |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 | 2050.11 | 2040.24 → 2039.66 | 2044.8850000000002 | 2026-11-11T09:14:01.052320+08:00 | 50.526316 | 2026-11-11T09:14:05.976327+08:00 | 40.956938 | 0.134928 | 51.0831 / 44.1361 |
| 49236051 | 2050.11 | 2040.24 → 2039.66 | 2044.8850000000002 | 2026-11-11T09:14:01.047358+08:00 | 50.526316 | 2026-11-11T09:14:05.976481+08:00 | 40.956938 | 0.134928 | 51.0831 / 44.1361 |
| 46130058 | 2050.11 | 2040.24 → 2039.66 | 2044.8850000000002 | 2026-11-11T09:14:01.061066+08:00 | 50.526316 | 2026-11-11T09:14:06.792808+08:00 | 41.339713 | 0.1311 | 51.0831 / 47.7392 |

The initial ETH extreme2040.24 extended causally to2039.66. Touch occurred near 50.53% retracement; subsequent turn from 2045.35 confirmed more than 8% of the 10.45 impulse. Final BTC confirmation was 44.14–47.74 signed bp in the short direction, above 24bp. The 5-day/20-day drifts were−5.453%/−14.541%; they are causal but do not predict the next 15 minutes. Full opening timestamps, guards, prices, source context and arm transitions are retained in JSON.

Confirmation semantics: the deployed helper requires the deepest touch in 50.0%–61.8%, then permits the confirmation/writer quote in 40.0%–61.8% after an 8% turn (macro_pullback.rs:161–164, 209–241). These confirmations at 40.956%–41.340% pass that implemented rule; they are not confirmations inside 50.0%–61.8%. Earlier shorthand can obscure this distinction. Any proposal to tighten the confirmation zone is a policy change requiring separate evaluation, not evidence of a PR219 regression.

## §6. Executable tick path, excursions and risk overrun

| Account | Paired quotes | Observed MFE R | Observed MAE R | Largest gap s | SL slippage USD/ETH | Initial risk USD | 0.6% budget USD | Realized loss % equity |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 | 2055 | 0.06938 | -1.004625 | 9.296278 | 0.56 | 149.8266 | 149.9134 | 0.6307 |
| 49236051 | 2056 | 0.083256 | -1.012951 | 9.30309 | 0.35 | 4.8645 | 4.9225 | 0.6119 |
| 46130058 | 2052 | 0.083256 | -1.012951 | 9.302162 | 0.63 | 153.0696 | 153.1163 | 0.6348 |

Minimum retained executable ask after entry was2042.08. No recorded trade approached+0.70R, much less+1R. Maximum retained spread was2.42 versus2.00 at entry. Stop fills occurred0.35–0.63 above the short stop; this explains the additional3.2–5.8% of R. Initial size did not exceed0.6% of admission equity. Gaps up to9.30seconds mean the continuous unobserved price path cannot be certified; no claim is made that every broker quote was retained. A0.80R profit-lock proposal would not have activated on the observed path.

## §7. BTC refusals and consumed-trigger reconciliation

Live was active, not paused: its two BTC arms and one ETH arm ended in recorded >61.8% deep-retracement invalidations before consumption. No live trigger was dispatched. The JSON retains these paths; demo confirmations do not prove identical live-feed sequencing.

| Account | Attempt SGT | Entry key | Final refusal | Peer origin / midpoint |
| --- | --- | --- | --- | --- |
| 47790949 | 2026-11-11T10:26:44.358307+08:00 | s3:47790949:10028:1794355200:1 | ENTRY_PEER_CONFLICT | 2050.11 / 2052.34 |
| 49236051 | 2026-11-11T10:26:44.375461+08:00 | s3:49236051:10028:1794355200:1 | ENTRY_PEER_CONFLICT | 2050.11 / 2052.34 |
| 46130058 | 2026-11-11T10:26:44.372521+08:00 | s3:46130058:10028:1794355200:1 | ENTRY_PEER_CONFLICT | 2050.11 / 2052.34 |

All three BTC attempts at 10:26:44 were refused before send. ETH midpoint2052.34 was above its2050.11 origin and therefore opposed the proposed BTC short. There is no BTC position ID, filled volume or realized loss to assign to these attempts. Six consumed triggers reconcile to six individual outcomes: three ETH fills and three BTC refusals; zero unmatched consumption. This provides live evidence for PR219 outcome attribution beyond its tests.

## §8. Hypothesis verdicts and ranked causes

A_redeploy_prereg_hash_reentry: DISPROVED for this window: registration SHA256 unchanged at 0444a4...7534c; one ETH filled entry per demo in 08:00 window, no second fill. Code hashes registration text, not deployment commit.

B_sequential_pair_double_dip_after_sl: POLICY CAPABILITY CONFIRMED, SECOND LOSS DISPROVED: occupancy lock releases after closure; BTC sibling attempted10:26 after ETH 09:29 stops, but all three final peer guards refused. No compounded second filled loss.

C_pr219_carried_arm_peer_defect: NOT SUPPORTED by filled trades: all three ETH arms originated09:08 in same08:00 session; no carried arm; peer BTC origin 08:00 and fresh final signed move44.14–47.74bp. This cohort does not independently test future carried-arm rollover.

D_sizing_or_slippage_breach: Initial price-risk sizing passed0.6% in all three; actual losses exceed1R by3.2–5.8% due adverse stop fills0.35–0.63USD/ETH beyond initial stops. A risk budget is not a guaranteed broker stop fill.

E_market_reversal_regime: SUPPORTED: valid short confirmation followed a brief downward impulse; ETH reversed upward and hit original stops about15min later. Recorded executable MFE only0.069–0.083R; no retained+0.70R observation. This is one economic signal replicated across three demos.

Rank1: short-horizon reversal after a valid but unsuccessful entry. Rank2: adverse stop execution increased loss beyond initial price risk. Rank3: three correlated demo copies amplify aggregate dollar loss but are not three independent confirmations of edge. Sequential pair re-entry is a policy exposure for future evaluation; it did not compound filled losses here. No deployment regression is established by these receipts.

## §9. Entire morning BTC/ETH market path

Symbol BTCUSD — representative demo46130058 retained two-sided quotes; final bucket ends12:55, not13:20.

| Bucket start SGT | Quotes | First mid | Min mid | Max mid | Last mid |
| --- | --- | --- | --- | --- | --- |
| 2026-11-11T07:50:00+08:00 | 6821 | 73674.23 | 73550.64 | 73757.59 | 73593.24 |
| 2026-11-11T08:20:00+08:00 | 7155 | 73608.19 | 73424.52 | 73701.09 | 73499.49 |
| 2026-11-11T08:50:00+08:00 | 7397 | 73499.33 | 73252.03 | 73529.74 | 73430.49 |
| 2026-11-11T09:20:00+08:00 | 7883 | 73430.81 | 73378.39 | 73969.225 | 73763.26 |
| 2026-11-11T09:50:00+08:00 | 7418 | 73763.32 | 73303.43 | 73792.71 | 73346.96 |
| 2026-11-11T10:20:00+08:00 | 7943 | 73348.37 | 73095.84 | 73725.86 | 73645.52 |
| 2026-11-11T10:50:00+08:00 | 7747 | 73645.47 | 73440.565 | 73822.83 | 73710.71 |
| 2026-11-11T11:20:00+08:00 | 6980 | 73726.835 | 73601.25 | 73776.39 | 73667.56 |
| 2026-11-11T11:50:00+08:00 | 7060 | 73667.35 | 73404.11 | 73826.755 | 73423.24 |
| 2026-11-11T12:20:00+08:00 | 7491 | 73423.09 | 73381.45 | 73799.59 | 73673.35 |
| 2026-11-11T12:50:00+08:00 | 1224 | 73673.41 | 73644.47 | 73787.02 | 73659.85 |

Symbol ETHUSD — representative demo46130058 retained two-sided quotes; final bucket ends12:55, not13:20.

| Bucket start SGT | Quotes | First mid | Min mid | Max mid | Last mid |
| --- | --- | --- | --- | --- | --- |
| 2026-11-11T07:50:00+08:00 | 3186 | 2050.1 | 2046.22 | 2052.51 | 2049.51 |
| 2026-11-11T08:20:00+08:00 | 4064 | 2049.5 | 2046.51 | 2055.88 | 2049.3 |
| 2026-11-11T08:50:00+08:00 | 4048 | 2049.1 | 2039.66 | 2050.68 | 2045.87 |
| 2026-11-11T09:20:00+08:00 | 5210 | 2045.88 | 2043.71 | 2068.46 | 2058.96 |
| 2026-11-11T09:50:00+08:00 | 4538 | 2059.02 | 2045.05 | 2060.93 | 2048.25 |
| 2026-11-11T10:20:00+08:00 | 4828 | 2049.32 | 2042.2 | 2061.6 | 2058.27 |
| 2026-11-11T10:50:00+08:00 | 4240 | 2058.28 | 2053.925 | 2066.895 | 2062.965 |
| 2026-11-11T11:20:00+08:00 | 4244 | 2063.23 | 2056.88 | 2065.49 | 2059.07 |
| 2026-11-11T11:50:00+08:00 | 4323 | 2059.06 | 2054.52 | 2066.72 | 2054.96 |
| 2026-11-11T12:20:00+08:00 | 5206 | 2054.95 | 2054.02 | 2069.47 | 2065.37 |
| 2026-11-11T12:50:00+08:00 | 852 | 2065.36 | 2062.44 | 2067.91 | 2062.49 |

## §10. Non-crypto activity and noon readiness

No oil/index/metal order was filled in the window. The following includes every non-crypto recorded arm/touch/invalidation/expiry in the selected state categories; exhaustive per-account veto/reason counts are in JSON. Repeated diagnostic counts are not distinct opportunities. Recovered Crude references remain86.755 from original07:00:07.xx quotes; Brent09:00 reference94.723. At12:00 crypto briefly recorded missing-opening diagnostics while recorder data became available; current references are restored (BTC73702.36, ETH2061.09) using04:00:00.026–.028UTC quotes, with no12:00 fill. A transient diagnostic alone does not prove a persistent preparation failure.

| Account | Symbol | Event SGT | Reason |
| --- | --- | --- | --- |

| Symbol | Kind | Reason | Records |
| --- | --- | --- | --- |
| NAS100 | s3_pullback_assessment | impulse_below_48bps | 24 |
| NAS100 | s3_pullback_assessment | vetoed_by_5d_20d_macro_trend | 24 |
| NAS100 | s3_session | session 2026-11-11 opening reference and macro prepared: open 27787.75; sigma checked per mark | 4 |
| NAS100 | s3_skip | human_ownership_book_requires_refresh | 4 |
| SpotBrent | s3_preopen | base preparation ready; unavailable later coefficients remain blocked | 4 |
| SpotBrent | s3_pullback_assessment | impulse_below_48bps | 4 |
| SpotBrent | s3_pullback_assessment | opening_reference_unavailable | 4 |
| SpotBrent | s3_pullback_assessment | vetoed_by_5d_20d_macro_trend | 4 |
| SpotBrent | s3_session | session preparation incomplete: opening_midpoint_unavailable: no valid recorded two-sided quote in first 15 seconds (native_open_first_quote_v18425_15s) | 4 |
| SpotCrude | s3_pullback_assessment | impulse_below_48bps | 4 |
| SpotCrude | s3_session | session 2026-11-11 opening reference and macro prepared: open 86.755; sigma checked per mark | 4 |
| SpotCrude | s3_skip | human_ownership_book_requires_refresh | 3 |
| US500 | s3_pullback_assessment | impulse_below_48bps | 128 |
| US500 | s3_pullback_assessment | vetoed_by_5d_20d_macro_trend | 124 |
| US500 | s3_session | session 2026-11-11 opening reference and macro prepared: open 7532; sigma checked per mark | 4 |
| US500 | s3_skip | human_ownership_book_requires_refresh | 3 |
| XAGUSD | s3_pullback_assessment | vetoed_by_5d_20d_macro_trend | 3 |
| XAGUSD | s3_session | session 2026-11-11 opening reference and macro prepared: open 51.5595; sigma checked per mark | 3 |
| XAGUSD | s3_skip | human_ownership_book_requires_refresh | 5 |
| XAGUSD | s3_skip | pair_risk_reservation_active | 3 |
| XAUUSD | s3_pullback_assessment | impulse_below_48bps | 3 |
| XAUUSD | s3_session | session 2026-11-11 opening reference and macro prepared: open 4034.015; sigma checked per mark | 2 |
| XAUUSD | s3_session | session 2026-11-11 opening reference and macro prepared: open 4034.01; sigma checked per mark | 1 |
| XAUUSD | s3_skip | human_ownership_book_requires_refresh | 6 |
| XAUUSD | s3_skip | yield_to_human_position_or_order | 4 |

## §11. Options for review — none implemented

Freeze one pair-level post-stop cooldown candidate for offline replay. Benefit: Can limit sequential correlated exposure. Trade-off: Can miss genuine later reversals; not demonstrated to improve this morning because BTC was already refused.

Calibrate stop-fill loss tails using observed broker spread/slippage by asset/session. Benefit: Separates intended price risk from realized loss distribution. Trade-off: Requires a larger independently measured sample; reducing size reduces dollar loss, not prediction error.

Evaluate one short-horizon reversal/regime filter against unchanged baseline on unspent dates. Benefit: Targets entry failure rather than profit-lock symptom. Trade-off: Adds selection/overfitting risk; predeclare rules and matched costs before evaluating.

Keep exit-lock candidate offline. Benefit: Avoids treating absent+0.7R excursion as an exit-policy failure. Trade-off: Other trade cohorts may differ; this sample cannot select a global exit rule.

Next evidence checks if authorised: (1) freeze one hypothesis and unchanged baseline; (2) reproduce this exact stopped-trade cohort with recorded spread and slippage; (3) evaluate separately on untouched dates with regime and pair-level loss accounting; (4) require positive net expectancy/payoff and the existing per-account goal checks before deployment. No parameter should be selected merely to remove these three copied losses.

## §12. Verification and handover

Verified Decimal closing cash−$324.60;3/3 broker entry/exit volume pairs;3/3 initial risk checks;3/3 final peer/turn checks; unchanged registration hash;6/6 consumed outcomes; history polls beyond cutoff. Accounts remain live 3/20 and demos 8/20 net wins, versus required15/20. Production unchanged. PR deliberately unmerged. Only three doc files are committed; serial/paragraph/by-model trace is embedded here instead of modifying log/ under the owner’s doc-only instruction.

JSON contains all position/order inventory, trade anatomy, exact source receipts and aggregate decision evidence. Unknown per-position unrealized values and continuous extrema remain null/unverified. No new broker query, position action or strategy change was performed.
