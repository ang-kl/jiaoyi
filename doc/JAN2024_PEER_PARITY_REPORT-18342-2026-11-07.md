# January 2024 peer collection and measured evaluation

№18,342 §§1–10 · 07-11-2026 21:08:59 SGT (+08:00) · by Codex (GPT-6), effort not exposed.

## §1. TLDR

**January macro: 2/6 net wins, mean -0.288994R, PF(R) 0.167304.**

**Cumulative S3 after deduplication: 6/20 closed-trade milestone; 3 wins, 3 losses, mean -0.188924R.** This pooled reused-development count is not the per-strategy/account73% / reference risk:reward1:1.38 goal. Qualified progress towards that goal remains unproven. No financial PASS or Stage2 promotion.

Ten peer jobs attempted; 10 completed, 10 contain both tick sides. 14/15 primary slots replayed. **US50024January remains a retained failed source**; it was not recollected. All five Crude slots retain inherited failed-M30-refetch protocol flags. Existing primary prices, preparation and Rust strategy rules were unchanged.

## §2. Measured pair results at24bp/side

| Primary / peer | Slots | Entries | Closes | Wins | Losses | Censored | Mean net R | PF(R) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| US500/NAS100 | 5 | 0 | 0 | 0 | 0 | 0 | unavailable | unavailable |
| XAUUSD/XAGUSD | 5 | 1 | 1 | 0 | 1 | 0 | -0.588491 | 0.000000 |
| SpotCrude/SpotBrent | 5 | 5 | 5 | 2 | 3 | 0 | -0.229095 | 0.233212 |

### Matched controls and prior macro

| Model | Entries | Closed | Wins | Mean net R | PF(R) |
| --- | --- | --- | --- | --- | --- |
| prior_macro_same_slots | 5 | 5 | 2 | -0.229095 | 0.233212 |
| s3_two_stage | 1 | 1 | 1 | 0.754547 | unavailable |
| macro_180m_two_stage | 6 | 6 | 2 | -0.288994 | 0.167304 |
| always_long_control | 4 | 4 | 2 | -0.099389 | 0.769343 |
| own_momentum_fixed_horizon | 4 | 4 | 1 | -0.480744 | 0.181012 |

## §3. What changed, and what did not

Only the missing NAS100 and XAGUSD peer observations were added to US500/gold replay payloads. Existing Crude/Brent inputs were reused. Macro requires six completed M30 bars in the same causal window; S3 has no peer filter. Exact S3 model-output parity passed for all14 replayed primary slots. Adding peers therefore did **not** manufacture new S3 trades.

The submitted text was truncated mid-review-block and assigned all wins, losses, class rows and returns as constants. Its collector arguments were unsupported; failure became`simulated_ok`. Those statements were replaced by actual sources and native fills. A1.024R loss is not a24bp-per-side model: cost/R depends on entry/exit prices and original risk. An entry-price stop is gross-flat, usually net-negative after costs.

The existing strict C<R screen, +1R next-valid-quote stop amendment, gross+1.38R target, broker bracket priority, native deadlines and censoring remain. Bid/ask includes spread once; assumed24bp per side is charged to actual fills, with the existing zero additional commission assumption. These costs and historical contract terms remain uncalibrated; no claim of realistic verified total costs is made.

## §4. Data continuity and cumulative accounting

The10 peer jobs align requested windows to the primary windows (US indices14:30UTC in January; metals07:00UTC). Each job performs bounded bid/ask pagination and one bounded M30 read. Ten logical jobs are not ten API messages. Matching dates/windows does not mean identical tick timestamps or zero quote gaps. The JSON retains actual peer bar coverage, bid/ask counts, maximum within-side gaps, native quality counts, failed primary slots and receipts.

Cumulative S3 combines the measured №18,331 and №18,338 results with this run, keyed by symbol, entry time and side. Repeated identities require identical entry/exit/net outcomes; disagreements stop publication. **1 duplicate observation(s) removed.** Earlier losses and protocol flags remain. There is no automatic addition of the two trades asserted in the supplied script.

## §5. Reproduction and next decision

[Frozen plan](../docs/jan2024-peer-plan-18342.json) · [Measured JSON](../docs/jan2024_peer_parity_results_18342.json). The JSON contains source/code/binary hashes, per-slot models, source failures, cumulative identities, controls and timing evidence. Raw sources remain outside Git and PostgreSQL. There were no new primary reads, paid model calls, live orders, risk changes or worker changes. Validation19August–5September2025 stays sealed.

Assess the measured before/after macro results and net costs before proposing another bounded experiment. Peer availability is an engineering improvement, not proof of an entry edge. Do not extrapolate a small pooled sample to the account goal or expand sampling until a favourable win count appears.

## §6. Post-run assessment: distinguish missing inputs from signal failure

Four US500 primary tick files (22,23,25,26January) contain **zero M30 bars** after eight failed preparation components each. Its24January primary is a retained failed source. Although the new NAS100 peer jobs contain the required bars, all five equity slots remain unusable for a complete primary/peer signal evaluation. Zero US500 entries is therefore not evidence that a fully prepared equity signal failed. These primary preparation failures were retained, not recollected or relabelled as clean evidence.

Compared with the same prior primary slots, peer data added exactly one macro entry: **XAUUSD-2024-01-23**, net **-0.588491R**. The five unchanged Crude trades supplied both January winners and retain inherited source-protocol flags. Across six closes, mean gross return was **+0.113153R**; assumed costs consumed **0.402147R**, leaving mean net **-0.288994R**. This small mixed-quality cohort does not establish a deployable edge. Costs must be measured, not reduced merely to obtain a positive result.

No new S3 close was created by adding peers. The repeated January Crude winner was deduplicated; the previous May loss remains. Cumulative progress is **3wins/6unique closes**, four censored entries, mean **−0.188924R** and PF(R)**0.637193**. Neither the20-close milestone nor the per-account73% goal is met.

The next data issue is the failed US500 preparation, not a lack of NAS100 peers. Investigate the retained failures before proposing any separately bounded source repair. The largest observed same-side peer update interval was136.441seconds; that alone does not prove missing ticks, but it rules out the supplied claim of universally continuous updates. This assessment changes no native outcomes, frozen rules, source files or financial qualification.
