# Independent AI review of №18,270

№ 18,273 §§1–5 · 06-11-2026 10:03:42 SGT (+08:00)
By Codex (GPT-6); effort setting not exposed.

## §1. Start here and download individual files

Give the other AI the [independent review prompt](Independent_AI_Review_Prompt.md),
the [response being reviewed](reviews/Response_18270.md), and the
[reconsidered assessment](reviews/Reconsidered_Review.md).
Each file can be opened on GitHub and downloaded with **Download raw file**, or
viewed with **Raw** and saved. No ZIP is committed, as instructed by the owner.

| Reference | Purpose |
|---|---|
| [Earlier response №18,269](reviews/Response_18269.md) | Exposes the earlier position for comparison and bias review. |
| [Initial handoff review](reviews/Handoff_Review.md) | Original findings before the fresh-data reconsideration. |
| [Revised agent instructions](instructions/1-Pr_Agent_Prompt_2026_11_06.md) | Owner-supplied proposal to assess, not automatically execute. |
| [Revised predictive plan](instructions/2-Predictive_Optimization_Plan_2026_11_06.md) | Owner-supplied empirical claims and proposed next steps. |
| [Reconsidered results JSON](reviews/fresh-reconsideration.json) | Fresh diagnostic rows, source hashes, per-symbol results and controls. |
| [Initial verification JSON](reviews/handoff-verification.json) | Earlier calculation and qualification findings. |
| [Original cost-context results](data/cost-context-results.json) | Original 480 diagnostic rows and recorded source/code provenance. |
| [Portable reproduction script](scripts/reproduce_review.py) | Offline calculation with explicit source-directory arguments. |
| [Reproduction receipt](reproduction-check.json) | Local check of this script against the saved review result. |
| [File hashes](SHA256SUMS.json) | Integrity of this documentation set, excluding this manifest itself. |

## §2. What is and is not included

This is a documentation and evidence-delivery change. Reports and attachments
retain their historical timestamps and describe their state at preparation;
their statements that no PR had been created apply to that earlier review.
The prompt's delivery instructions have been adapted for individual repository
files. This README records the subsequent publication requested by the owner.

The **raw 60 episode files and tick streams are not included**. Neither the
14.2MB ZIP nor duplicate Rust source trees are added. The JSON references contain
derived diagnostic results, not a replacement for the raw source corpus.
The earlier review verified 40 August episodes plus 20 November episodes. Another
AI can inspect the included calculations, claims and code references immediately,
but cannot independently reproduce raw-quote outcomes without those corpora.
It must mark unavailable verification explicitly, not claim it ran.

## §3. Frozen code references

Reviewed main was `64739b45a65b285595e60ae679354cc015f499fd`. Use that revision
when comparing the original claims; distinguish subsequent source changes.

- [Cost-context calculations](https://github.com/ang-kl/jiaoyi/blob/64739b45a65b285595e60ae679354cc015f499fd/ops/research/cost_context.py)
- [Collection validation](https://github.com/ang-kl/jiaoyi/blob/64739b45a65b285595e60ae679354cc015f499fd/ops/research/market_collection.py)
- [Class scoring](https://github.com/ang-kl/jiaoyi/blob/64739b45a65b285595e60ae679354cc015f499fd/ops/research/class_research.py)
- [Milestone scoring](https://github.com/ang-kl/jiaoyi/blob/64739b45a65b285595e60ae679354cc015f499fd/ops/research/milestone.py)
- [Strategy 3 accounting detail and Kelly](https://github.com/ang-kl/jiaoyi/blob/64739b45a65b285595e60ae679354cc015f499fd/crates/jiaoyi/src/strategy3.rs)
- [Cost reserve definition](https://github.com/ang-kl/jiaoyi/blob/64739b45a65b285595e60ae679354cc015f499fd/crates/jiaoyi-core/src/strategy3/mod.rs)
- [Qualified outcomes](https://github.com/ang-kl/jiaoyi/blob/64739b45a65b285595e60ae679354cc015f499fd/crates/jiaoyi-store/src/outcomes.rs)

## §4. Offline reproduction when the original corpora are available

Python 3.10+ and its standard library are sufficient. Replace the three
placeholder paths with the original source checkout and data directories:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 docs/ai-review-2026-11-06/scripts/reproduce_review.py \
  --repository /path/to/reviewed-jiaoyi-checkout \
  --development /path/to/original-development-episodes \
  --fresh /path/to/recent-collection-data \
  --output /tmp/jiaoyi-independent-review
```

The development directory must contain the 40 filenames/hashes in the frozen
cost-context plan. The recent directory must contain `collection-plan.json`,
`collection-summary.json`, and its 20 original symbol job files. The script
checks these identities and reconstructs quotes from local data only. It does
not fetch broker data, build the trading service, require credentials or call a
model API. Missing files cause failure rather than substituted evidence.

Output: `fresh-reconsideration.json` in the selected output directory. Source
hashes, tables, controls, quotes and rows should match the included review JSON;
`generated_at_utc` and `script_sha256` differ because the calculation was rerun
with a portable wrapper. Reproduction with the same algorithm does not itself
validate the methodology independently; inspect raw-quote arithmetic as the
review prompt requests. The explicit path arguments/output destination are the
only calculation-wrapper changes from the original review script.

## §5. Scope and goals

No production implementation, strategy change, account permission, cost
assumption, risk limit or worker setting is activated by these files. No rule is
promoted by a diagnostic win rate. The original 73% / 1:1.38 per-strategy/account
goal and interim research target remain unchanged. The local reproduction
receipt is separate from the PR's required CI results; CI status is recorded on
the PR itself. Time is Money: this PR delivers review material, not another
research search or live trading experiment.
