# Strengthening validation: 30 October 2026

**Document:** VALIDATION-20261030-STRENGTHENING · **Prepared:** 30-10-2026, 20:55 SGT.
**Source:** owner “proceed to strengthen”, acknowledged № 17,919 §1 (20:30 SGT); implementation and verification on source base `e2bf32941ac03916e844c9b8befaee942aa949b4`. By <Codex (GPT-6), effort: not exposed>.

[§VALIDATION.1] This is a bounded engineering evidence record, read with the [current system contract](current-system-contract.md). The statuses below are a preparation-time snapshot, not release or financial acceptance. Update them only from actual check and deployment receipts.

| Area | Evidence at preparation | Remaining gate |
|---|---|---|
| cTrader transport cancellation and write handling | Transport suite: **74/74 passed**. | Complete required CI and verify the deployed release; a local pass does not establish broker behaviour. |
| Python coordinator release-base and release-scope checks | Final local suite: **157/157 passed**, including the two further root-agent regressions. | Required CI and deployed coordinator behaviour remain pending. |
| Account actor lag and recovery | Actual S3 overflow regression failed before the fix and passed afterwards: 5,120 quotes, at least 1,024 actor gaps, zero observer gaps, exactly one TP1 and no entry. | S3 58/58, S1 38/38 and monitor 5/5 also pass; formatting and Clippy pass without suppressions. Deployed diagnostics remain pending. |
| Current architecture and dividend instructions | Current contract and draft dividend contract written; historical registrations retained. | Dividend routine remains disabled pending the connector and confirmed routine. |
| CI and deployment | First CI passed both image/engineering gates but exposed a false TCP-readiness test. Controlled reproduction and child-specific readiness correction pass all four startup tests; every assertion is retained. | Required checks, reviewed merge identity, component deployment identity and affected behaviour. |
| Concurrent trading writers | No cross-process exclusion proof supplied by these repairs. | Shared-writer ownership/fencing and safe handover require separate evidence before competing executors. |
| Economic outcome | No new qualified trade-window result established by these changes. | Per-strategy/account latest-20 goal, realised costs, out-of-sample robustness and portfolio validation remain separate gates. |

## Branch protection: observed gap and proposed configuration

[§VALIDATION.2] GitHub branch metadata reports `main` as unprotected, and the applicable branch-rules response is `[]`. The detailed branch-protection GET returned HTTP 403 because the integration lacks Administration access. That denied read is an inspection limitation; it is not itself proof of a particular legacy protection setting. No protection change has been applied.

[§VALIDATION.3] The following is a concrete proposal for the branch-protection endpoint, pending suitable Administration access and configuration readback. It requires the observed GitHub Actions app (`15368`) checks against an up-to-date base, permits the owner's existing merge-without-peer-review policy, enforces the rules for administrators, and prohibits force pushes and deletion:

```json
{
  "required_status_checks": {
    "strict": true,
    "checks": [
      {"context": "check", "app_id": 15368},
      {"context": "docker", "app_id": 15368},
      {"context": "engineering", "app_id": 15368}
    ]
  },
  "enforce_admins": true,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
```

[§VALIDATION.4] This JSON is **proposed, not applied**. The owner has authorised necessary strengthening and release; the current limitation is missing Administration access, not an additional approval requirement. Do not bypass the integration's permissions. The final read of `main` followed by a merge retains a race: another writer can advance `main` between those operations. Local base checks narrow that gap but do not close it. Server-enforced strict checks or an equivalent verified merge policy need their own applied-state receipt.

[§VALIDATION.5] PR138 first run `41059420383` failed an existing startup account-row assertion. The old readiness predicate can accept a foreign listener before this child registers accounts; a controlled database lock and listener reproduced that failure. The original runner collision is not independently observed. The test now uses `PORT=0`, its own child stdout serving event and bounded cleanup. Local startup 4/4, formatting and strict Clippy pass; the corrected head requires all CI gates again. Final merge/deployment receipts belong in the PR description.
