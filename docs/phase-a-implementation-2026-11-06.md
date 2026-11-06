# Phase A: evidence gates and bounded Kelly history reads

№18,278 §§1–5 · 06-11-2026 11:28:21 SGT (+08:00)
By Codex (GPT-6); effort setting not exposed.
Owner authority: proceed with №18,277; reconciles the Gemini Phase-A prompt
with the verified source receipts and the actual outcome-reader schema.

## §1. Changes and boundaries

- Both research scorecards inspect missing-data status, marks and zero-sided
  streams before ignoring non-entry episodes. A reason is counted once per
  episode, even if repeated in marks. Valid `flat_predictor`, `peer_disagreement`
  and `no_breakout` skips remain distinct. The native
  `no_next_fresh_fill_quote_before_deadline` reason is included.
- Reuse analysis preserves `plan-sha256.txt`, requires the explicit
  `source_collection_plan_sha256`, and writes `analysis-plan-sha256.txt`.
  Only the original entry-rule identifier can differ in a reused request;
  account, symbol, dates, costs and quantities must still match. The existing
  native verifier continues checking the saved source and original study.
- Both scorecards expose a descriptive normal one-sided 90% IID lower-bound
  ranking score. Class selection ranks passing candidates by that score,
  mean net R, then rule name. All sample-size, censoring, expectancy and
  per-symbol requirements remain. This is a variance-sensitive heuristic,
  not a confidence guarantee for correlated, adaptively selected trials.
- Kelly avoids the expensive outcome/history read when too few positions can
  possibly qualify. Fewer than 20 entry keys triggers a bounded lookup of at
  most 20 distinct fill/reconciliation position receipts; zero keys needs no
  lookup. A key may identify multiple positions in the outcome schema, so key
  count alone is not a proven upper bound. At 20 keys or 20 possible positions,
  the existing qualified reader runs with HashSet key membership. Its admission
  formula, threshold and positive-fraction sizing behaviour remain unchanged.
- `history_queried=false`, `history_status` and null `n`/`wins` distinguish
  unqueried history from measured zero. The pure peer-coherence helper rejects
  non-finite input and aggregate overflow; no production signal calls it.

Realised accounting, `close_result`, broker-exit cost handling, zero-spread
reconstruction, native volume defaults, production universe, live permissions,
registration, risk settings and historical/recent collection boundaries are
unchanged. Phase A implements no oil/metals/index candidate or promotion.

## §2. Corrected provenance and exploratory plan

The immutable original collection-plan SHA256 is:
`8c8cb376d2bbe25a61567c55cb6187dd2616c2b6fd08baaaefda608f597e88ac`.
The different full hash quoted in Gemini's prompt is not the saved receipt.
The original native quantities are US500=100, JPN225=1000, and
AUS200/GER40/NAS100/UK100/US30=10; these are native units, not lots.

[The Phase-A analysis plan](phase-a-research-plan-2026-11-06.json) reuses the
original 40-source development corpus and the six existing ablation candidates
without modifying earlier frozen plans. It declares collection and parent
analysis lineage separately. It is `reuse_only`: collection, including validation
collection, is prohibited. Earlier reuse plans without an explicit collection
hash must be migrated to a separate corrected analysis plan rather than having
their receipt overwritten to pass a comparison.

Use a separate output directory containing the original source jobs and original
collection receipt; do not overwrite a previous analysis's result files:

```bash
python3 ops/research/class_research.py analyse --stage development \
  --plan docs/phase-a-research-plan-2026-11-06.json \
  --output /path/to/separate-phase-a-output \
  --binary /path/to/class_prediction_study
```

## §3. Five acceptance groups

1. Rust Kelly check proves the full reader is skipped below the threshold even
   when its deals table is unavailable; verifies 0/19/20 boundaries and the
   one-key/multiple-position safeguard.
2. Rust coherence check covers unanimous/mixed signs, zeros, empty input,
   NaN/infinity and overflow, including the expected mean.
3. Python scorecard checks cover all emitted gap reasons through status/marks/
   zero streams, deduplication, genuine skips, and variance-sensitive ranking
   without permitting an insufficient one-trade candidate.
4. Python receipt checks reject wrong hashes, preserve original receipt bytes,
   write the analysis digest, and retain quantity/cost/account/date matching.
5. Optional-corpus Python reproduction checks reconstruct 480 old and 240 fresh
   diagnostics, including 1,281,012 fresh ticks and 222 observed/18 missing-peer
   rows, and compare every saved field except generation time and wrapper hash.

The fifth check runs locally with `JIAOYI_RESEARCH_DEVELOPMENT` and
`JIAOYI_RESEARCH_FRESH` pointing to the original corpora. Without those variables,
including on ordinary CI, it reports an explicit skip and never downloads data.
The other acceptance checks run normally in CI. Local verification and the
required exact-head CI are recorded separately in the PR and journal.

## §4. Evidence and limitations

The local research suite passed all 17 tests with both corpora configured. The
original fixed-horizon diagnostics reproduced unchanged. Analysis of all 40
development sources and six existing replay candidates reported zero passing
class/candidate screens, while surfacing missing first-hour samples and missing
next-fill evidence explicitly. A stricter scorecard is an engineering repair,
not an improvement in financial performance or proof of reaching 50%.

No raw tick corpus or ZIP is added. Exact output/receipt hashes and verification
counts are recorded in the accompanying verification JSON. The original 73%
net-win / 1:1.38 per-strategy/account goal remains unchanged.

## §5. Follow-on work

Full SL/TP/trailing candidate replay is a separate implementation. Oil own/peer
controls, metals reversal and index momentum require causal features, explicit
initial risk, executable-side triggers, cost sensitivities and visible censoring.
Fixed-horizon direction success cannot substitute for that evaluation. Freeze
new candidates before opening further validation data; Phase A does not reopen
validation or activate a trading rule. Time is Money: retain proven corrections
and use the repaired evidence path for the next bounded experiment.
