# Component-specific engineering release verification

Trace: №18,018–18,025 §§1–2; 31 October 2026, 14:10–14:31 SGT; by <Codex (GPT-6), effort: not exposed>.

The coordinator previously expected a trading deployment for every merged job.
Documentation and journal changes are excluded by the trading service's watch
patterns, so valid repository-only work could remain in deployment wait forever.

`ops/engineering/releases.py` now classifies the complete immutable Git diff
against the pinned parent/merge configuration and Railway's effective watch
patterns. Unsupported or inconsistent evidence receives an explicit unresolved
disposition. Protected paths, append-only journals, exact-head CI and the existing
single-writer and mutation-window rules remain enforced.

Repository-only jobs verify exact approved file hashes without a trading restart
or model session. Approval must be supplied by an operator or independent review;
model output is not itself an approval receipt. Missing substantive acceptance
remains `verification_pending`, rather than being reported as complete.

Runtime jobs require exactly one active successful deployment at the merge SHA.
A newer failed build does not obscure a still-active correct deployment. Failed,
crashed, unexpectedly skipped, superseded and expired deployment waits retain the
candidate and an explicit reason. The wait deadline is 30 minutes from GitHub's
immutable merge time; restarting the coordinator does not reset it. Read-only
reconciliation continues after the owner's mutation cutoff. No push, merge,
deployment, strategy change or extra session authority is created by this rule.

The status report exposes the release backlog across all job ages, including
counts and at most 20 concise identities/reasons. Material transitions are stored
in the existing events table, with no new telemetry table or per-poll event rows.
Eligible read-only verification can proceed beside a held release; competing
engineering writers cannot.

## Registering exact repository acceptance

An authorised operator runs:

```sh
python3 /app/worker.py accept-repository --job JOB_ID --receipt /private/receipt.json
```

The bounded regular receipt file must belong to the coordinator user, with no
group/other access (for example mode 0600). Its exact JSON keys are:

```json
{
  "head": "FULL_TESTED_HEAD_SHA",
  "source": "independent_review",
  "criteria": "exact_artifact_content",
  "reference": "Review identity and acceptance criteria",
  "files": {
    "docs/example.md": "SHA256_OF_EXACT_FILE_BYTES",
    "journal/YYYY-MM-DD.md": "SHA256_OF_EXACT_FILE_BYTES"
  }
}
```

`owner_approved_artifact` is the other accepted source. A deletion uses `null`.
All changed paths must match at verification; omitted or extra paths fail.
A new receipt reopens the same pending candidate for reconciliation. An identical
receipt is deduplicated. This does not infer approval, bypass CI, deploy anything,
change the work window or reserve a model session. Private production credentials
and raw customer data do not belong in this file or this repository.

## Validation and release boundary

The full Python suite passes 176 tests, including real-Git scope checks,
repository acceptance ingress, immutable deadlines, active/failed deployment
selection, all-age backlog, fairness and existing authority controls. Required
`check`, `docker` and `engineering` CI, independent review and actual deployed
source readback are separate release gates. Their receipts are retained by the
release operator and linked in the PR closure.

This change establishes engineering release status, not trading profitability.
The RR 1:1.38 and at least 73% net wins over the latest 20 qualified closed trades
per strategy/account remain unproven. No strategy, risk limit, owner pause, worker
allowance, authentication route or expired mutation deadline is changed.
