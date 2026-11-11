# Wednesday London review and New York preparation

№18,431 §§1–11 · 11 November 2026 · Cutoff 19:45 SGT · by Codex (runtime model/effort not exposed; Extra High was requested).

## TLDR

**S3 traded and lost: six new demo positions, zero winners, −$619.25 during 12:55–19:45 SGT. Day-to-cutoff S3 cash is −$943.85.** All six positions closed; no live S3 entry was sent. The three demos remain at 8/20 latest net wins; live remains 3/20 on an older cohort. Financial qualification fails.

All 18 consumed triggers have individual outcome receipts: six fills and twelve pre-send peer refusals. Crypto 20:00 preparation is 8/8 and NY 22:30 preparation 24/24. London opening evidence is 22/24 with the two live metal slots intentionally withheld by human/pair guards; crypto 16:00 is 8/8. London preparation has two unavailable late coefficients, so “fully ready all session” would overstate the evidence.

This audit changed no trading code, settings, orders or deployment. Production was verified on `aebf404dedf89dbcdb8cee78c95a582eea042d51`. Publication/merge status is established by the linked PR, not predicted in this report.

## §1. Scope, timing and evidence

The exact activity interval is **[11 November 12:55, 19:45) SGT**, continuing №18,427. Day totals use [00:00,19:45). The environment recovered after cutoff, so historical deal/intent/decision rows are filtered to the requested interval. Health and account books were collected around 20:12–20:14 SGT and are separate observations. No later trades are included in window cash. SQL ran read-only; no new historical market collection or strategy experiment was performed. The machine-readable companion contains trade receipts, guard evidence, individual trigger/outcome joins, preparation rows, opening evidence and source SHA256 hashes.

## §2. Account reconciliation

| Account | Post-cutoff balance USD | Float USD | Equity USD | Window S3 USD | Day S3 USD | Snapshot SGT |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 501.84 | 5194.32 | 5696.16 | 0.00 | 0.00 | 2026-11-11T20:11:53.401000+08:00 |
| 46130058 | 25082.80 | -19.68 | 25063.12 | -303.83 | -465.82 | 2026-11-11T20:11:52.919000+08:00 |
| 47790949 | 24543.59 | -13.17 | 24530.42 | -307.05 | -464.64 | 2026-11-11T20:11:53.125000+08:00 |
| 49236051 | 807.02 | 0.00 | 807.02 | -8.37 | -13.39 | 2026-11-11T20:11:52.975000+08:00 |

Account-snapshot margin level is **not verifiable** for nonempty books because broker aggregation is unverified; no sum of per-position margins is substituted. The empty small-demo book has no used-position margin, so a finite margin percentage is not meaningful. The separate owner-assessment current-margin checks are shown below as derived observations, not reconciled account-margin certificates.

| Account | Derived current-margin check | State |
| --- | --- | --- |
| 43002148 | 1187.8631159677184 | pass |
| 46130058 | 50307.34644720996 | pass |
| 47790949 | 6158.480618598112 | pass |
| 49236051 | None | pass |

The live account’s pending-entry stress projection is a breach in the observer. This is a risk diagnostic, not authority to cancel human orders. Floating gains on human positions are not S3 realised profit.

## §3. Open inventory and human protection

| Account | Kind / ownership | ID | Symbol | Side | Native volume | SL | TP |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 | position / human/external | 649992251 | JPN225 | buy | 4000 | 56571.3 | None |
| 43002148 | position / human/external | 649772867 | JPN225 | buy | 12000 | 56450.7 | None |
| 43002148 | position / human/external | 637075128 | NatGas | buy | 150000 | 2.327 | 3.875 |
| 43002148 | pending_entry / human/external | 400465199 | XAUUSD | buy | 1000 | 3877.3 | 4339.75 |
| 43002148 | pending_entry / human/external | 400937857 | JPN225 | buy | 5000 | 59366.5 | None |
| 43002148 | pending_entry / human/external | 398403207 | NatGas | buy | 200000 | 2.381 | 4.275 |
| 43002148 | pending_entry / human/external | 400083120 | JPN225 | buy | 15000 | 55679.5 | None |
| 43002148 | pending_entry / human/external | 400920027 | NatGas | buy | 100000 | 2.423 | None |
| 46130058 | position / s1 | 260823431 | AUS200 | buy | 60 | 8819.5 | None |
| 46130058 | position / s1 | 261231902 | AUDUSD | sell | 900000 | 0.69978 | None |
| 47790949 | position / s1 | 260743517 | AUS200 | buy | 60 | 8819.5 | None |
| 47790949 | position / s1 | 261231903 | AUDUSD | sell | 900000 | 0.69973 | None |

No S3 position was open in the post-cutoff snapshot; no S2 inventory was identified. The two large demos each retain S1 AUS200 and AUDUSD positions. Six S1 trailing amendments occurred in the window (three per large demo); these are not new trades or S3 interference.

Live 43002148 retains three human positions and five pending entries. No live S3 events/intents or external closing deals were found in the selected window. NatGas and JPN225 remain permanently reserved. XAUUSD is dynamically blocked by the human pending order; XAGUSD is blocked by sibling-pair occupancy. **The metals are not permanently human-reserved symbols.** Label ownership remains `jiaoyi-s3`; no recorded bot interference is supported by the inspected receipts, rather than a blanket 100% guarantee.

## §4. Complete afternoon S3 ledger

All entries below are shorts; native volume is cTrader volume in hundredths of a unit, not assumed broker lots. Initial SL and TP come from entry/protection receipts.

| Account / position | Symbol | Entry SGT / price | Exit SGT / price | Native volume | Initial SL / TP | Net USD | Net R |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 47790949 / 261629338 | BTCUSD | 17:03:33.033 / 73722.11 | 18:31:09.756 / 74076.94 | 45 | 74047.45 / 73273.14 | -159.67 | -1.090620 |
| 49236051 / 261629441 | BTCUSD | 17:03:43.091 / 73753.51 | 18:32:29.623 / 74075.48 | 1 | 74078.85 / 73304.54 | -3.22 | -0.989734 |
| 46130058 / 261629442 | BTCUSD | 17:03:43.076 / 73753.51 | 18:32:29.615 / 74075.48 | 46 | 74078.85 / 73304.54 | -148.11 | -0.989667 |
| 46130058 / 261636289 | SpotCrude | 17:14:20.985 / 86.383 | 17:16:39.746 / 86.841 | 34000 | 86.831 / 85.764 | -155.72 | -1.022321 |
| 49236051 / 261636412 | SpotBrent | 17:14:37.517 / 94.546 | 17:18:04.173 / 95.19 | 800 | 95.156 / 93.704 | -5.15 | -1.055328 |
| 47790949 / 261636417 | SpotBrent | 17:14:38.559 / 94.531 | 17:18:00.284 / 95.135 | 24400 | 95.141 / 93.689 | -147.38 | -0.990191 |

All closes are protective broker exits consistent with the original stop. Two BTC fills and one Brent fill improved on their stop levels; this does not make them discretionary exits. The application’s stop-loss classification is a nearest-level proxy. There were no S3 breakeven/trailing amendments on these six positions.

| Position | Observed favourable R | Signed worst R | Original cash risk USD | Quote rows / max gap s | Confirmation→guard ms | Order→fill ms |
| --- | --- | --- | --- | --- | --- | --- |
| 261629338 | 0.810168 | -1.095131 | 146.4030 | 22019 / 14.088 | 17.332 | 590 |
| 261629441 | 0.906682 | -1.039835 | 3.2534 | 22315 / 14.091 | 25.008 | 544 |
| 261629442 | 0.906682 | -1.039835 | 149.6564 | 22315 / 14.095 | 18.679000000000002 | 535 |
| 261636289 | 0.000000 | -1.022321 | 152.3200 | 305 / 3.835 | 24.462999999999997 | 530 |
| 261636412 | 0.068852 | -1.031148 | 4.8800 | 407 / 5.372 | 17.322 | 516 |
| 261636417 | 0.044262 | -1.000000 | 148.8400 | 393 / 5.404 | 13.778 | 542 |

No retained quote reached +1R, so breakeven acknowledgement latency is **N/A**, not zero or a missed acknowledgement. Crude’s best retained ask-side return was −0.084821R; favourable excursion is displayed as zero and the signed raw value is retained in JSON. The largest BTC recorder gap was about 14.1 seconds, so the observations do not prove that no unrecorded price touched +1R.

## §5. Trigger, veto and execution accounting

| Measure | Recorded count |
| --- | --- |
| New arms | 45 |
| Consumed triggers | 18 |
| Individual terminal outcomes | 18 |
| Persisted opening intents | 18 |
| Filled entries | 6 |
| Peer <24bp refusals before send | 12 |
| Consumed triggers missing outcome | 0 |
| Unknown final guard receipts | 0 |
| Deep >61.8% invalidations | 19 |
| Pair reservation state transitions | 4 |
| Pair reservation diagnostic rows including skips | 16 |

Each consumed decision ID joins exactly once to `consumed_decision_id` in its outcome. Each demo had six opening attempts, two fills and four peer refusals; live had no opening intent. Diagnostic rows are deduplicated state reports, not tick counts or independent missed trades. The 19 invalidations must not be added to the 18 consumed triggers as though they were fills.

After BTC stopped at 18:31–18:32 SGT, ETH sibling attempts at approximately 18:51:58 and 19:13:51 SGT were refused on all three demos by the <24bp peer guard. Pair occupancy prevents simultaneous exposure; it is not a same-session loss cooldown. No further crypto fill occurred before cutoff. The maximum recorded confirmation-to-final-guard latency was 25.008 ms; this is distinct from order-to-broker-fill latency.

No consumed arm crossed a session rollover in this window. The carried-arm/current-peer repair is therefore **not exercised by this consumed-attempt sample**. A null regression count in the review JSON is intentional; zero would imply evidence that this window does not supply.

## §6. Opening references and preparation

| Window | Evidence | Qualification |
| --- | --- | --- |
| London 15:00 | 22/24 historical origins; 2 live metal slots withheld | 11 contemporaneous arm/session quote receipts plus later health historical-origin evidence for remaining active slots |
| Crypto 16:00 | 8/8 persisted opening quotes | BTC origin 73991.55 live / 73991.34 demos; ETH 2066.44 |
| Crypto 20:00 | 8/8 complete pre-open records; 8/8 opening quotes observed after cutoff | BTC 73914.53; ETH 2055.39 |
| NY 22:30 | 24/24 complete preparation records before cutoff | Opening quotes not yet due; preparation is not permission to trade |

London’s 24 preparations reported base readiness, but marks 420 and 450 minutes each had only 7 same-slot endpoints against 14 required. Earlier marks were usable; later unavailable marks remain blocked. This is a preparation coverage limitation, not evidence that all London trading stopped. Crypto 20:00 preparation was last completed around 16:00:27 SGT; NY preparation around 15:00:36 SGT. Both precede their T−10 deadlines.

The quote source tag `recorded_ticks_native_open_6s` remains valid for captures within the original six seconds; the repaired 15-second tag applies to later permitted captures. Its presence alone is not a regression. Opening quotes and their original bid/ask/observed timestamps are included in JSON.

## §7. Latest-20 objective

| Account | Net wins / 20 | Win rate | Realised payoff | Net-R PF | Latest close UTC |
| --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 | 15.0% | 3.366963 | 0.594170 | 2026-10-29T22:24:36.655Z |
| 46130058 | 8/20 | 40.0% | 1.509341 | 1.006228 | 2026-11-11T10:32:29.615Z |
| 47790949 | 8/20 | 40.0% | 1.379972 | 0.919981 | 2026-11-11T10:31:09.756Z |
| 49236051 | 8/20 | 40.0% | 1.518121 | 1.012081 | 2026-11-11T10:32:29.623Z |

The discrete 20-trade target is at least 15/20. No account qualifies. Demo 47790949 payoff is 1.379972, slightly below 1.38 before rounding. Live’s last close was 29 October, so its latest-20 does not validate the current policy. Demo wins stayed 8/20 because incoming losses replaced older losses; unchanged win count does not mean no trading or no deterioration. Reused overlapping latest-20 windows are not independent evidence.

## §8. Interpretation and next investigation

**Observed:** BTC gained roughly 0.81–0.91R before reversing; oil barely moved favourably and stopped within roughly two to three-and-a-half minutes. Entry guards accepted direction, pullback and peer evidence, yet this sample lost. Engineering receipt completeness and profitable entry selection are different acceptance criteria.

**Inference:** the BTC failure deserves a frozen, offline horizon/exit counterfactual; the oil failure deserves event-local entry-quality and peer persistence analysis. A lower profit-lock threshold chosen after these losses would be an overfit proposal, not a verified repair. The previously rejected lock remains rejected for production. Six losses are two correlated market episodes replicated across demos, not enough independent evidence to select a new rule.

**Next review:** (1) replay the exact pre-entry/exit quotes against unchanged rules and record missing-tick sensitivity; (2) compare matched prior winning and losing BTC/oil episodes with parameters frozen; (3) report per-market net expectancy after evidenced costs and adverse fill tails; (4) evaluate any candidate on separate dates before proposing a production change. No experiment or rule change was executed in this read-only audit.

The three demos all entered, including the small account: minimum-lot/margin refusal was not the cause of these six outcomes. Do not increase risk or weaken human/pair guards to make the activity count larger.

## §9. Evidence limitations

- Historical deal polling ended seconds before the cutoff; no atomic broker snapshot exactly at 19:45. Stored deals fetched later were filtered by execution time.
- Book, equity and health snapshots are post-cutoff and explicitly timestamped.
- MFE/MAE use retained executable ask quotes for short positions; recorder gaps mean they are observed extrema, not a complete exchange tick tape.
- Stop classification is supported by protective exit/fill geometry; a distinct broker SL-versus-TP reason is not present in the decoded close receipt.
- Current margin values differ in evidential strength: audited account snapshots cannot verify aggregation, while owner assessment exposes a derived current-margin check.
- 3.74 bp/side is an owner-selected cost allowance, not verified round-trip calibration.
- No consumed cross-window arm in this window; carried-arm rollover correctness cannot be certified by this sample.
- Six account trades represent two closely related market episodes across three demo accounts; they are not six independent strategy experiments.

## §10. Publication and production isolation

Only this Markdown report and its HTML/JSON companions belong to the change. Railway watch patterns exclude `doc/**`; the earlier report merge left deployment `c0fedaca-0370-4cec-9234-f59f8c7cafce` on `aebf404d`. The report PR must pass `check`, `docker` and `engineering` before squash merge. Final PR/merge and post-merge deployment evidence will be returned in the completion response. No report text claims that an unobserved deployment or merge has succeeded.

## §11. Gemini review instructions

Read this report first, then the same-name JSON. Check: six-trade cash sums, individual trigger/outcome join, full-day versus window cutoffs, strict SL classification limits, London late-coefficient gaps, current snapshots versus historical activity, and latest-20 cohort age. Treat `financial_pass:false` as binding evidence status. Do not interpret the report as authorisation to deploy a new entry/exit policy. The JSON carries source hashes and full per-trade evidence for independent arithmetic checks.
