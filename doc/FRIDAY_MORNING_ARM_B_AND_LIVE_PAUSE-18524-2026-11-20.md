# Friday morning Strategy 3 release and account retirement

**No. 18524 and No. 18525 · 20 November 2026 · Singapore time**

## 1. Release decision

The two continuing accounts are Live **43002148** and Demo **46130058**. The overlapping broker-book snapshots at 08:03:45 SGT showed **zero open Strategy 3 crypto positions** on those accounts. Demo 46130058 held one Strategy 3 Silver position, #265256567. Its position management continues.

On the remaining demo, Strategy 3 can open positions in Gold, Silver, WTI, Brent, US500 and NAS100. Its BTCUSD and ETHUSD entry switch defaults off. On Live, new S1 and S3 entries also default off. A deposit, a margin improvement or removal of a manual order alone cannot enable live execution. The financial qualification remains false.

## 2. Closing deals

The audit covers **[05:27:14, 08:04:11.664729) SGT**. Three broker closing deals totalled **−$44.58**. Post-PR #214 cumulative Strategy 3 closing cash across the four historically monitored accounts was **+$1,741.89** through this cutoff.

| Account | Position / closing deal | Symbol | Exit time SGT | Entry | Exit | Broker or strategy receipt | Net USD |
|---|---|---|---|---:|---:|---|---:|
| 47790949 | 265223568 / 335553390 | SpotBrent short | 05:57:00.911 | 94.013 | 94.228 | Session-end close | −35.05 |
| 46130058 | 265220046 / 335561365 | BTCUSD short | 06:22:08.440 | 61,897.33 | 61,925.61 | Broker stop-loss | −8.47 |
| 47790949 | 265219265 / 335561378 | BTCUSD short | 06:22:10.688 | 61,932.74 | 61,939.31 | Broker stop-loss | −1.06 |

Both Bitcoin shorts had already closed when the 08:00 session expired. Brent's session-end request was sent at 05:57:00.506601, and its fill receipt arrived at 05:57:01.031184. The four-account event ledger recorded zero `MARKET_CLOSED` rejections in the audit interval.

## 3. Retirement handoff

| Broker account | Positions visible at 08:03:45 SGT | Local management after release |
|---|---|---|
| Live 43002148 | External NVDA.US and NatGas | Connected. Existing ownership protections apply. |
| Demo 46130058 | Six S1 positions and one S3 Silver position | Connected. S1 and traditional S3 continue under their existing risk gates. |
| Demo 47790949 | Six S1 positions and S3 Silver #265256568 | Retired at startup, including its local trailing-stop and scheduled-exit workers. |
| Demo 49236051 | S1 ETHUSD #262796763 | Retired at startup, including local S1 management. |

The owner submitted the two broker closure requests and expressly authorized stopping local management with positions still present. Local retirement sends no liquidation or order cancellation. Settlement of the remaining demo inventory belongs to Pepperstone. The JSON lists all position IDs and the observation time of each book.

## 4. Entry controls and evidence

| Control | Behaviour |
|---|---|
| `JIAOYI_S3_ENABLE_CRYPTO_4H` | Explicit `true` enables S3 crypto entry evaluation. Otherwise the actor skips arming, admission preparation and dispatch with `CRYPTO_4H_DISABLED_ARM_B`. |
| `JIAOYI_ENABLE_LIVE_EXECUTION` | Explicit `true` permits new automated live entries to reach the existing gates. Otherwise S1 and S3 record `LIVE_EXECUTION_PAUSED_PRE_QUALIFICATION`. A live crypto S3 entry would require both switches. |
| Protection | Reconciliation and exit management run independently of the entry switches. Existing stop, breakeven, target and session-end rules continue on managed accounts. |
| Retirement | Both retired IDs are removed from the startup allowlist, demo aliases and declared environments before the worker and refusal plans are built. Startup marks their existing account rows inactive so database-backed research and health readers use the same roster. Historical ledger records remain available. |
| Guard accounting | Idle policy skips use the existing deduplicated decision record. A refusal after a consumed S3 trigger uses its individual linked terminal receipt. The trigger-status view exposes the policy code. |

Both switches were unset or empty on the running service before release. The read-only ledger matched both consumed triggers to exactly one outcome, with zero unknown entry-guard receipts and zero recorded live S3 interference. Net cash uses broker closing profit and swap less commission and conversion fee at the recorded precision. Open-position counts use private, complete broker-book assessments, cross-checked against the S3 actors.

## 5. Verification

The release starts from production commit `52ea64ca0850758e13be8fd945b691f023b88049`. The regression suite exercises default and opt-in permissions, prepared dispatch, the session-end close of a carried crypto position, retained live S1 protection, the retired-account startup path and individual outcomes. Local formatting and workspace Clippy passed. The full workspace test run passed 822 tests with 1 already marked ignored. A separate parser check recognized both refusal codes. The 08:21 SGT active-account book recheck again found zero open S3 crypto positions. The release requires the three CI jobs and a fresh active-account drain check before merge. Runtime verification checks the deployed SHA, active actor IDs, entry-gate reasons and the account refusal counter. [Measured audit JSON](FRIDAY_MORNING_ARM_B_AND_LIVE_PAUSE-18524-2026-11-20.json)
