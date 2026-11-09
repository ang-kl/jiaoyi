# TL;DR — Trading resumed, but financial recovery is unproven

№18,411 §§1–12 · 10 November 2026 · by <Codex (GPT-6), effort: not exposed>. Review window: 9 November 19:47 → 10 November 06:49:35 SGT. The owner corrected the initially stated evening cutoff to the Railway morning clock.

Six S3 demo positions closed: three ETHUSD wins and three SpotCrude losses, net −$206.95 in paper accounts. The live account had no new S3 entries; its recorded non-S3 closing cash was −$2.12. Two new S1 AUS200 positions remained open. All four trading actors were connected, unpaused and processing quotes. This is functioning execution with losing economics, not general inactivity.

The clearest newly evidenced readiness defect is a current-session M30 dependency: the loader reports “holiday, skipped” despite an original opening quote, then commonly becomes ready only about 31 minutes after the open. A second defect is an obsolete policy-specific monitoring parser that rejects all six current entry-validation receipts. A concurrent-history assessment race also intermittently makes one demo’s goal assessment unknown. No production code, risk limits, orders or deployment were changed for this report.

The latest-20 S3 win rates remain 15%, 15%, 15% and 25%, far below 73%. Two demos improved by 5 percentage points from the prior checkpoint, one was unchanged at 15%, and live was unchanged. A 50% hit rate over only two new trades per demo does not qualify the research milestone or establish an edge.

## §1. Scope, source authority and cutoffs

The starting checkpoint is the verified PR #212 rollout, not the later chat answering whether environment publication was needed. Production remained at commit 6fdfcf1d013829815e5101e7042468f83f5157f9, Railway deployment c3d33986-d1a0-4306-8de3-7de058b191b8 (SUCCESS). PR #211 restored all native index sessions; PR #212 exposed prepared-admission refusals. No subsequent mainline code change was present when this investigation began.

Sources: read-only repeatable-read PostgreSQL queries; stored cTrader deals, intent/fill receipts and decoded broker SL/TP acknowledgements; /health; owner setup assessments; recorded supervision and preparation decisions; actual source code at the deployed commit. No verification trade, historical broker collection, Telegram send or live amendment was made. Broker execution time defines the trade cutoff; later database reads may retrieve already-executed receipts.

History receipts are successful through approximately 06:49:05 SGT, about 29 seconds before the requested cutoff, and were fetched at 06:49:06. Their complete=false field means the live window is unsealed, not failed pagination: history.rs and goals.rs explicitly distinguish this. Health returned at approximately 06:49:42. These are bounded observations, not an atomic 06:49:35 broker snapshot.

## §2. Account performance and open exposure

All amounts below are USD. Balance, floating P&L and equity are actual NY-close observations at 10 November 06:00:07 SGT; interval realised cash runs through the 06:49 cutoff. Balance changes reconcile to the interval closing cash. Percentage is realised cash / observed starting balance at 9 November 19:45:06, not a time-weighted equity return. Live money and demo money are intentionally reported separately.

| Account | Balance | Float | Equity* | Interval realised | % starting balance | S3 entries / closes | Price-only stop exposure** |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 43002148 LIVE | 716.16 | 4,091.35 | 4,807.51 | -2.12 | -0.295% | 0 / 0W 0L | 5,527.36 |
| 46130058 DEMO | 24,516.98 | -4.98 | 24,512.00 | -105.41 | -0.428% | 2 / 1W 1L | 136.92 |
| 47790949 DEMO | 24,236.85 | -54.51 | 24,182.34 | -99.07 | -0.407% | 2 / 1W 1L | 212.07 |
| 49236051 DEMO | 794.77 | 0.00 | 794.77 | -2.47 | -0.310% | 2 / 1W 1L | 0.00 |

* Equity is balance plus broker net unrealised P&L, excluding unverified credit/bonus treatment. ** Stop exposure is a supplemental mark-to-stop price projection; it excludes unquoted exit charges, gaps and slippage. Full cash open risk is therefore unknown for non-empty books. These figures are not guaranteed maximum losses.

Live: balance $716.16, floating profit $4,091.35, equity $4,807.51. Demo subtotal: balance $49,548.60, float $-59.49, equity $49,489.11; realised S3 loss $206.95. A combined paper-plus-real total would obscure the actual capital at risk.

Live risk needs attention independently of S3 performance: the 06:00 price-only mark-to-stop exposure was about $5,527.36, exceeding the $4,807.51 cash-equity estimate, with four pending entries also present. The later live-oil admission cache reports negative stop-stressed margin. This is a reason to examine the human/external book, not a reason to relax the margin floor. No human position or pending order was changed.

| Account | Positions visible near cutoff | Attribution / relevance |
| --- | --- | --- |
| 43002148 LIVE | 2 × JPN225 long; NatGas long; NVDA.US long; 4 pending entries | No evidenced S1/S3 ownership on these objects. S3 must leave them alone. Pending XAUUSD also blocks S3 metal-pair participation. |
| 46130058 DEMO | AUS200 long, position 260743514 | S1 opened 10 Nov 03:42 SGT; not an S3 trade. |
| 47790949 DEMO | AUS200 long 260743517; older AUDUSD short 259680976 | S1 positions. AUDUSD quote no longer supports its original entry benchmark; registered exits remain authoritative. |
| 49236051 DEMO | No open positions | No residual S3 exposure in the observed book. |

The live −$2.12 is closing deal cash on position 651937484, broker symbol ID 11432; no owned S1/S3 order provenance was found in this extraction. The retained symbol catalog did not resolve its name. It is reported as non-S3/unattributed cash, not as a bot loss or as proof of who pressed the button. Its earlier mixed-side lifecycle is not converted into a clean bot trade count.

## §3. Every new S3 closed position

| Demo account / position | Symbol | Entry → exit SGT | Sell fill → buy close | Net USD | Net R | Observed exit |
| --- | --- | --- | --- | --- | --- | --- |
| 46130058 / 260655209 | ETHUSD | 10 Nov 00:59:10 → 10 Nov 04:00:01 | 2,131.990 → 2,126.730 | 44.97 | 0.3046 | Session close |
| 49236051 / 260655210 | ETHUSD | 10 Nov 00:59:10 → 10 Nov 04:00:00 | 2,132.020 → 2,126.730 | 1.43 | 0.3067 | Session close |
| 47790949 / 260655217 | ETHUSD | 10 Nov 00:59:11 → 10 Nov 04:00:00 | 2,132.020 → 2,126.730 | 44.65 | 0.3063 | Session close |
| 47790949 / 260741238 | SpotCrude | 10 Nov 03:38:54 → 10 Nov 04:40:41 | 86.359 → 87.364 | -143.72 | -0.9882 | Broker protective stop |
| 46130058 / 260741239 | SpotCrude | 10 Nov 03:38:54 → 10 Nov 04:37:53 | 86.256 → 87.279 | -150.38 | -1.0210 | Broker protective stop |
| 49236051 / 260741495 | SpotCrude | 10 Nov 03:39:13 → 10 Nov 04:38:13 | 86.328 → 87.303 | -3.90 | -0.9731 | Broker protective stop |

All six are fully matched single opening and closing fills with equal quantities and distinct position/deal identities. Cash uses gross profit + swap − absolute closing commission − absolute conversion fee at broker money precision. Those closing cost components are zero in these six receipts. Net R divides actual closing cash by original filled-share price risk × deposit conversion. It is not the hypothetical 3.74 or 24 bp research repricing.

ETHUSD contributed +$91.05; SpotCrude contributed $-298.00. Six account trades are only two highly correlated market setups. The size-weighted demo cash profit factor is 0.306; mean account-trade net R is -0.3441. Neither establishes independent statistical significance.

## §4. Why the wins were small and the losses larger

ETHUSD: the short agreed with negative 5/20-session drift, retraced into the touch zone, then confirmed a turn with a falling BTC peer. It moved favourably but never reached the configured +1R breakeven threshold in the recorded ask path. The best observed ask was 2,114.88 versus entry prices 2,131.99–2,132.02 and R = 17.27. That is approximately +0.991–0.992R, narrowly short of +1R. Price then retraced, and the 04:00 SGT session close realised only +0.305–0.307R. The configured +1.38R TP was not hit.

SpotCrude: the short also passed its macro, turn and peer gates. Price initially fell, with a best recorded ask of 85.451, then reversed to the protective stops. Maximum favourable excursions were approximately +0.803R, +0.893R and +0.875R across the three account fills. None reached +1R; there was no rule-authorised breakeven change. Closing losses were −1.021R, −0.988R and −0.973R. Stop-fill differences are consistent with executable-price slippage around the broker stops; accepted brackets and closing receipts are present.

| Account / symbol | Initial SL | Initial TP | Best observed favourable R | Ask observations | Largest ask-update gap |
| --- | --- | --- | --- | --- | --- |
| 46130058 / ETHUSD | 2,149.260 | 2,108.150 | 0.9907 | 37263 | 00:00:06.580938 |
| 49236051 / ETHUSD | 2,149.290 | 2,108.180 | 0.9925 | 37226 | 00:00:06.523437 |
| 47790949 / ETHUSD | 2,149.290 | 2,108.180 | 0.9925 | 37260 | 00:00:06.588158 |
| 47790949 / SpotCrude | 87.376 | 84.955 | 0.8928 | 7085 | 00:00:10.831667 |
| 46130058 / SpotCrude | 87.258 | 84.873 | 0.8034 | 6854 | 00:00:10.85688 |
| 49236051 / SpotCrude | 87.330 | 84.945 | 0.8752 | 6814 | 00:00:10.824484 |

Inference: a full-loss / small-session-win distribution can lose money even at a 50% hit rate. This sample’s winners were approximately 0.306R while losses averaged approximately 0.994R. The exit geometry promises only an order target, not a realised 1.38 payoff. An earlier breakeven trigger might reduce giveback, but can also stop out future winners; it is a hypothesis requiring replay, not a proven remedy.

Limits: the tick cache is not a gapless exchange tape. ETHUSD had two ask-update gaps above six seconds; oil had seven, with the longest about 10.86 seconds. No +1R hit is present in retained quotes and no S3 breakeven amendment was recorded for these six positions. That supports the observed-rule explanation, while leaving unobserved intragap paths unknown. No news catalyst or institutional motive is asserted without an external source.

## §5. Did the execution repairs actually operate?

| Mechanism | Measured evidence | Assessment |
| --- | --- | --- |
| Touch then turn, fresh peer | All six writer receipts satisfy 50–61.8% touch, later ≥8% turn, 40–61.8% confirmation range and ≥24 bp same-direction peer. | Observed on real entries; not proof of economic edge. |
| Writer speed | Confirmation → final guard 6.334–18.232 ms; order → fill 505–610 ms. Touch → guard 2.03–22.43 seconds includes waiting for a fresh qualifying confirmation. | No freeze in these six paths. Timing scopes differ. |
| Protection / exits | All six received broker SL and TP acknowledgements. Three ETH closes used session-end market exits; three oil closes used broker protective orders. | Protection and exits are evidenced; no target hit or breakeven amendment in this cohort. |
| Pair reservation | Three SpotBrent attempts were refused before send for fresh peer conflict; no simultaneous filled Brent/Crude pair appears in these new S3 trades. Live XAGUSD also reports a metal-pair reservation. | No observed duplicate pair entry; not an exhaustive proof over all future race conditions. |
| Demo quantity / margin | New planned price risk was within the normal sizing budget; all six writer margin projections exceeded 250%. Broker volumes reconciled to deals. | No minimum-lot bypass is evidenced in this batch. Stops can still slip beyond planned cash risk. |
| Human ownership | Live manual/external positions and pending orders remain visible separately; XAUUSD yields and XAGUSD is pair-blocked. | No S3-labelled intervention on human positions was found. |
| Session rollover / restored references | Recorded references retain original first-six-second quote timestamps. New session readiness is separately delayed by the current-M30 dependency. | Previous redeploy preservation remains evidenced; this does not prove timely readiness at later opens. |

## §6. Why some accounts or instruments did not trade

Live S3: zero persisted opening intents, not a stuck unanswered order. Cached oil admission refusals describe stop-stressed margin below the warning threshold (about −150% in the latest cached oil examples). At the prior checkpoint, the live projected margin had instead been below the 250% floor. These are time-specific admission observations, not interchangeable current-margin measurements. The live book also contains a human XAUUSD pending order, with XAGUSD blocked by the same pair reservation.

There were nine demo S3 persisted opening attempts: six fills and three confirmed local SpotBrent refusals, with zero broker rejection or unresolved attempt in that cohort. A refusal is useful when the live peer no longer supports the entry. Repeated background veto records are material state changes, not independent missed trades; their count must not be presented as a count of profitable opportunities.

Indices, metals and BTC/ETH alternated between band/impulse conditions, macro-direction vetoes, peer checks, waiting pullbacks and session limits. Current state alone cannot explain every prior minute. The report therefore retains the interval decision groups and window preparation evidence in the JSON. There is no basis to flip a rejected long into a short merely because a later chart looks attractive; the opposite trade requires its own qualifying setup.

## §7. Newly evidenced defects and unresolved issues — priority order

| Priority | Finding and evidence | Proposed minimal next action | Acceptance check |
| --- | --- | --- | --- |
| P1 | Opening readiness still depends on a current M30 bar. strategy3.rs load_day rejects a valid recorded opening midpoint if first_volume is absent. Typical first successful preparation was ~31 minutes after open. | Use verified native calendar + original fresh opening quote + causal precomputed context to distinguish an active opening from unavailable forming history. Preserve fail-closed missing-quote and holiday checks. | Replay actual first-6-second opening receipts with empty current M30 response; eligible opening must become ready promptly. Missing/stale/protected data must still fail. |
| P1 reporting | dispatch_evidence.py hard-codes owner_18392 and expects bounded=true. Current owner_18409 receipts carry confirmed_turn instead. All six valid current receipts were marked invalid and latency became null. | Version the parser against actual typed receipt schemas; retain historical interpretation and reject unknown policy/hash combinations. | Use these six stored real receipts plus a mismatched/tampered receipt; recover correct confirmation timing without accepting wrong versions. |
| P2 reporting | Demo 46130058 latest S3 assessment becomes unknown when history changes during its multi-query assessment; preceding measured assessment has the same 3/20 window. | Use a consistent read snapshot or bounded retry, while retaining an explicit assessment timestamp and last measured result. | Concurrent history insert must not mix later evidence into an earlier cutoff or fabricate a financial PASS. |
| P2 evidence | S1 live and one demo retain incomplete historical ownership/lifecycle evidence. Other S1 samples are below 20. | Reconcile exact account-scoped opening quantities and provenance; keep unreconciled records unknown. | No human/netted increment silently counted as an owned strategy trade. |
| Research | Near-1R giveback and small forced-close winners can offset the apparent hit-rate improvement. | Freeze one bounded exit comparison; evaluate costs and forced/strict closes separately, then use untouched dates. | Assess net expectancy and payoff as well as win rate; no deployment from this two-setup sample. |

The opening defect is code-backed: load_day computes opening_midpoint from recorded ticks, then rejects it unless a returned session-open M30 bar has positive tick volume (strategy3.rs, around lines 1041–1080). production_sessions::band_at explicitly supports minutes <30 using the prior same-slot coefficient. Thus the delay is not simply the intended strategy waiting for its first 30-minute mark.

For demo 46130058, all twelve observed new session/symbol preparations first succeeded about 30.99–31.06 minutes after their open: six traditional NY symbols and BTC/ETH at three four-hour windows. Their original opening quote provenance is retained, so the holiday label is demonstrably misleading for these active windows. Live BTC’s 04:00 SGT window has its first successful persisted preparation only at 06:15:52; the longer delay needs its own trace before assigning a second root cause. Live metals are ownership-blocked and are not counted as unexplained missing preparations.

No claim is made that any blocked opening would have been profitable. Repair the readiness invariant, then test the economic effect. No automatic relaxation of macro filters, peer confirmation, the 250% margin floor, reserved symbols or human protection is recommended.

## §8. Progress towards the invariant goal

Goal: 73% net wins over the latest 20 completed trades per strategy per account, with the 1:1.38 objective retained. Since 20 trades have 5-percentage-point increments, the operational minimum is 15/20 (75%), not 14/20. Realised payoff is reported separately from the planned gross TP. A single qualifying window would still not establish consistency.

| S3 account | Latest 20 | Change since checkpoint | Gap to 15/20 | Realised payoff | R-based PF | Net R |
| --- | --- | --- | --- | --- | --- | --- |
| 43002148 | 3/20 (15%) | +0 pp | 12 more wins in a 20-trade window | 3.367 | 0.594 | -6.240 |
| 46130058 | 3/20 (15%) | +5 pp | 12 more wins in a 20-trade window | 0.847 | 0.149 | -10.949 |
| 47790949 | 3/20 (15%) | +0 pp | 12 more wins in a 20-trade window | 1.014 | 0.179 | -9.689 |
| 49236051 | 5/20 (25%) | +5 pp | 10 more wins in a 20-trade window | 1.102 | 0.367 | -7.863 |

These rolling windows include older policy versions; they do not qualify the new owner_18409 policy. The new-policy cohort has only two completed trades per demo and zero live entries. Demo 46130058 uses its last measured 06:44 SGT result; its newer 06:49 result is explicitly unknown because the receipt watermark changed. The numbers agree, but their validation states differ. Live’s last S3 close remains 30 October 06:24 SGT, so its unchanged 15% is not fresh evidence about this release.

| S1 account | Observed closed sample | Qualification |
| --- | --- | --- |
| 43002148 | 0/0 | unknown; 1 position lifecycle(s) lack complete net-R evidence |
| 46130058 | 0/4 | unknown; 2 position lifecycle(s) lack complete net-R evidence |
| 47790949 | 0/2 | insufficient_evidence; fewer than 20 completed trades |
| 49236051 | 0/0 | insufficient_evidence; fewer than 20 completed trades |

Measured improvement: +5 percentage points on demos 46130058 and 49236051. Demo 47790949 remains 15% and its rolling PF/payoff deteriorated. The account goal has not been reached; financial_pass=false. No aggregate score is used to conceal a failing account.

## §9. Risk, reporting and what the UI can establish

Near the cutoff, S3 owned positions and unresolved openings were zero on all accounts; fast-monitor exit counters had advanced since rollout. This establishes running loops at observed checkpoints, not guaranteed coverage every millisecond. The UI displays snapshots; browser visibility does not control broker trading or exits.

The whole-account observer has many unknown checks where market schedules or fresh quotes are unavailable, particularly at maintenance/closed sessions. Unknown is not a pass and should not be confused with an execution halt. The retained AUDUSD S1 short on demo 47790949 has a direction-benchmark breach; that is an observation, not authority to reverse or close the trade outside its registered exit policy.

Daily equity return, MTD/YTD return and 99% one-day VaR are not inferred from this 11-hour follow-up. Reliable comparison needs cash-flow-adjusted equity marks and an appropriate return sample. Slippage here is actual fill versus recorded decision quote; no VWAP benchmark is available. Full stop-loss cash exposure remains unverified where future costs are unquoted.

## §10. Proposed sequence and five decision checks

1. Repair and test opening-readiness semantics using these real cached receipts. Check both normal openings and genuinely unavailable evidence. Do not resample a later price as the opening reference.

2. Repair the versioned monitoring parser and assessment consistency. Validate six real current receipts, old-policy receipts and deliberately invalid evidence; publish timing with its actual definition.

3. Replay the same six trade paths under the unchanged production rules to establish entry/exit/accounting parity. Preserve costs and gaps. Only after parity freeze one exit hypothesis addressing near-1R giveback; do not optimise many thresholds on these losses.

4. Evaluate that frozen change on separate dates and report per asset/session and account sizing. Require positive net expectancy and an adequate sample, not merely 50% wins. Human/external trades and correlated demo copies must remain separate.

5. Before any approved release, pass required CI, verify the exact deployed commit, then observe the next native opening and first genuine trade end-to-end. Compare planned risk, broker protection, realised costs and latest-20 progress. Close the repair only on measured evidence; otherwise retain a named unresolved issue.

This report task implements none of those production changes. Existing trading authorization and the owner’s pause policy remain unchanged. There is no need to republish environment variables merely to read this report.

## §11. Verification and files for another AI

Report validation reconciled all six S3 lifecycles and cash totals; checked per-account balance deltas against closing-deal cash; verified the six turn/peer/margin receipts and broker protection; checked cutoff exclusion and deduplicated identities; distinguished policy-era goal metrics, actual marks and unknown evidence. No application test suite was rewritten for a report-only change. Required repository CI still gates the report PR merge.

Read this Markdown or its standalone HTML first, then the companion JSON. The JSON includes trade-level deals, decoded protection events, writer guards, tick-path summaries, opening readiness evidence, per-account goal windows, history receipts and source hashes. Raw source captures remain hashes for traceability; the compact evidence contains the facts required for this report’s calculations, not the entire production database.

Companion files: FOLLOWUP_TRADING_AND_REPAIR_REVIEW-18411-2026-11-10.json; FOLLOWUP_TRADING_AND_REPAIR_REVIEW-18411-2026-11-10.html. Deployed-source references: crates/jiaoyi/src/strategy3.rs (load_day and breakeven); crates/jiaoyi-core/src/strategy3/production_sessions.rs (early-session band support); ops/engineering/monitoring/dispatch_evidence.py (legacy receipt parsing); crates/jiaoyi-store/src/goals.rs (cutoff/watermark checks).

## §12. Instructions for Gemini / independent review

Review №18,411 against its companion JSON and deployed commit 6fdfcf1. First challenge the M30/readiness diagnosis using original opening quotes and first-success timestamps. Independently recompute closing cash, net R and the latest-20 deltas. Distinguish current-policy parser rejection from an invalid trade. Do not count the three demo copies as independent signals, turn the live float into realised bot profit, or treat unknown S1 provenance as a qualified strategy result. Recommend a minimal bug-repair scope separately from an exit-design experiment. No conclusion in this report authorises a new trading rule.
