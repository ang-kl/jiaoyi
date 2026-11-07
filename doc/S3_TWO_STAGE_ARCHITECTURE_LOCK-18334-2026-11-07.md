# Strategy 3 two-stage architecture decision and peer authorisation

№18,334 §§1–5 · 07-11-2026 17:50:08 SGT (+08:00) · by Codex (GPT-6), effort not exposed.

## §1. Decision and evidence

**The owner selects `s3_two_stage` as the mainline design direction and confirms the three peer pairs below. This PR records that decision. It does not activate a production strategy or establish financial qualification.** The offline implementation is already merged in [PR #183](https://github.com/ang-kl/jiaoyi/pull/183), commit `d551a50fed74ae8d44a845344159ea9c30d75b98` (not PR #182).

At the assumed 24 bp per side, the existing five oil closes have 3/5 net wins (60.0%), mean +0.12385077R and PF(R) 1.45149070. Three closes carry inherited source-protocol flags; two are winners that were censored in the baseline. The two closes without that particular flag average -0.00422097R and retain other quote-quality limitations. Index/gold have no closed trades in this arm. These reused development dates are not clean validation.

The dynamic gate arm produced 1/83 wins (82 losses), mean -3.035711R. This rejects that cost-gate replacement under the tested assumptions. It does not prove a universal 48 bp cutoff or identify volatility alone as the cause.

## §2. Locked interpretation of admission and exits

- Owner-selected minimum: `R_bps >= 48.0`, where `R_bps = 10000 * R_price / reference_price`.
- Retain the stricter **tested** admission: `R_price > median_spread + round_trip_commission + 2 * 24/10000 * reference_price`, checked at decision and dispatch. A bare `R_bps >= 48` drops spread/commission and permits equality; it is not equivalent and is not what produced the five-trade result. With non-negative costs at 24 bp/side, the tested rule already implies the minimum.
- Original stop at −1R; executable bid for longs / ask for shorts reaching +1R proposes an entry-price stop. It becomes effective on a strictly later valid quote; existing stop/target processing retains priority. Gaps use executable quotes. Whole-position gross target +1.38R, rounded outward; no extra trailing or partial exit in this arm. Native session deadlines and censored source boundaries remain.
- Entry-price breakeven is **gross**, not net. The experiment changes both trailing and target geometry. It does not isolate a trailing-only effect.
- There is **no guaranteed positive expectancy above 45% wins**. For the simplified frictionless binary +1.38R/−1R model, the threshold is `1/2.38 ≈ 42.02%`; with constant cost `c` in R it becomes `(1+c)/2.38`. At 45%, gross expectancy is just +0.071R before costs. Actual two-stage exits, flat exits and gaps require their full realised outcome distribution.

## §3. Authorised peers and actual availability

| Class | Primary | Peer | Frozen data / implication |
| --- | --- | --- | --- |
| Energy | SpotCrude | SpotBrent | Both present; existing offline mapping |
| Equity index | US500 | NAS100 | NAS100 absent; index macro confirmation unavailable |
| Metal | XAUUSD | XAGUSD | XAGUSD absent; gold macro confirmation unavailable |

These are owner-authorised research relationships, not proven predictive signals. Mappings already exist in PR #183's macro arm; this record does not add a peer filter to live S3 or collect missing observations. No fabricated peer bars or fallback substitution.

## §4. Next evaluation and goal tracking

Validation **19 August–5 September 2025 remains sealed**. Next work is a separately frozen, bounded multi-cohort study: predeclare representative dates, session coverage, actual peer availability, costs, cohort/job budget, failure handling and stopping rule before outcomes. Respect the spent-source registry; no duplicate reads disguised as new jobs. Preserve all attempted jobs, nonentries, failures and censoring.

Twenty closed trades is an interim sample milestone, not a qualification rule or permission to keep sampling until favourable. Report energy/index/metal separately and distinguish simulated research from account outcomes. The project goal remains **73% net wins / reference risk:reward 1:1.38 over the latest 20 closes per strategy per account** (at least 15 wins gives 75%). Qualified progress from this documentation run: **none demonstrated**; no new trades or market replays were run. No live promotion follows from the pooled 60% observation.

## §5. Traceability and release checks

The [JSON specification](../docs/s3_two_stage_architecture_lock_18334.json) retains the actual measured values and limitations. Evidence: [offline-repaired-engine-results-18331.json](../docs/offline-repaired-engine-results-18331.json), SHA256 `ca7fcbfbeee34f96f319345021037c3ef3651d52d47487f854fd9895779f0151`; [complete experiment report](OFFLINE_REPAIRED_ENGINE_REPORT-18331-2026-11-07.md). The source audit remains FAILED, affecting 20 episodes overall.

The accompanying script creates a task branch before artifact writes, stages only four named files, requires explicit success from `check`, `docker` and `engineering` on the unchanged PR head, and verifies merged file hashes. Missing/pending/null/failed checks cannot merge. The final review block is generated from the actual merge response. Timestamps are actual SGT recording times; journals preserve serial/paragraph/model references and consult GROK_Log.txt. No Rust runtime, broker setting, research results or deployment configuration is modified by this record.
