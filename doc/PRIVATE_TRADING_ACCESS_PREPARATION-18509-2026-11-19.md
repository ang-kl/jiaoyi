# Private Trading access preparation — №18,509

This change prepares a staged move away from a public dashboard. It does not activate security, remove domains, create a login service, change production settings, or deploy Trading. The default HTTP behavior remains `legacy` so the migration cannot be confused with a completed cutover.

## 1. Scope and baseline

The owner selected no detailed UI for now, with any future login/report viewer isolated from Trading. The implementation is based on `92802dc26f35f64d933a83ad0c44fa3334208bf1`; its checkout tree matches reviewed backup source `91deb19e916f6fb50eec39a0dd88f0df905c1ea1`. The locally inspected trading code matches production-reference `65a3ba7a2594f55cc75b33ccefddfdb3ff87dca2`. A later read-only Railway metadata check at 18:29:59 SGT verified one successful active deployment at that production-reference commit, created on 15 November. No account snapshot or Trading endpoint was requested.

The verified local backup contains the entire reviewed checkout and shared Git metadata, plus a portable Git bundle. The archive SHA256 is `0a1db7f7dd27cd6e963d2c3bfb80ad1926446fd61d31dc4a1073b153c5300d88`. Archive extraction, per-file hashes/modes, portable checkout and Git fsck passed. This is local code/Git recovery, not an off-machine, database, image or running-state backup. The private archive is not included in this PR.

## 2. Implemented boundary

`JIAOYI_HTTP_PROFILE=private` enables a machine-only HTTP boundary. No OTP, login/session service, new dependency, database auth call or authentication operation is added to quote, risk, order or exit processing.

| Private-profile caller | Allowed surface |
|---|---|
| Anonymous | GET/HEAD `/health`, minimal `{"status":"ok"}` before any handler, DB query or actor-map lock |
| Audit Bearer | Explicit allowlist of 17 JSON-read paths; no research submission or polling |
| Research Bearer | GET/HEAD/POST `/owner/backtests` and `/owner/market-data` only; all existing account/request/collection/job guards remain |
| Browser pages/assets | Five HTML/JS paths return 404; no interactive login is added |

The two machine secrets must be distinct printable ASCII strings, 32–4000 characters. Provision high-entropy random secrets; accepted length alone is not proof of entropy. Missing/invalid/equal credentials deny private data with 503 without stopping the application or minimal health. Basic and the legacy owner password do not bypass private mode. Unknown/invalid profile values fail closed. `X-Jiaoyi-Http-Profile` identifies private/disabled responses for explicit release verification.

The `legacy` profile intentionally retains existing behavior. Unset profile is also legacy. **Merging/deploying this code without explicitly selecting and verifying private mode does not secure the website.**

## 3. Existing-work compatibility

Seven Engineering callers now use shared validated transport: monitoring sources, source snapshots, market validation, stop-loss reviews, incident collection, release verification and worker-supplied evidence. Six research scripts use the same origin/credential selection without changing sampling, economic rules, seals, authority checks or job limits.

- `TRADING_BASE_URL` selects the origin; `JIAOYI_BASE_URL` is an alias. Conflicting values fail. Explicit research `--base` takes precedence.
- `JIAOYI_TRADING_ACCESS_MODE=legacy` is the compatibility default. `scoped` selects audit/research secrets explicitly and never falls back to the legacy owner secret.
- HTTPS retains normal CA/hostname verification. Plain HTTP is accepted only for the separately pinned exact `JIAOYI_TRADING_PRIVATE_ORIGIN`, with an explicit port and a `.railway.internal` host. Private HTTP bypasses ambient proxies; all redirects are refused.
- The pin is an approval mechanism, not evidence that private DNS, transport or IPv4/IPv6 reachability works. No production origin is changed here.
- Full diagnostics preserve their existing shape. Minimal liveness is rejected as account evidence. Release verification deliberately needs only liveness plus its existing deployment identity checks.
- Persisted `DEFAULT_TASK` and `ACCEPTANCE_TASK` strings remain byte-identical, preserving queued/accepted job identities. Only model-facing instructions change. Models receive bounded timestamped snapshots rather than service credentials. Automatic jobs reuse the latest matching evidence pair when fresh; otherwise the existing two-read snapshot mechanism replaces their previously requested direct reads. No new periodic poller or queue transition is introduced.
- Updating `oil_replication.py` changes its adapter hash. Old two-episode preflight receipts correctly stop further collection until the existing offline preflight is rerun against cached evidence. Historical receipts are not rewritten, and collection authority is not relaxed.

## 4. Configuration contract — names only

| Location | Configuration | Meaning |
|---|---|---|
| Trading | `JIAOYI_HTTP_PROFILE` | `legacy` compatibility or explicit `private` |
| Trading + audit callers | `JIAOYI_AUDIT_BEARER_TOKEN` | Audit-read authority only |
| Trading + research callers | `JIAOYI_RESEARCH_BEARER_TOKEN` | Existing bounded research operations only |
| Clients | `JIAOYI_TRADING_ACCESS_MODE` | `legacy` or explicit `scoped` |
| Clients | `TRADING_BASE_URL` / `JIAOYI_BASE_URL` | Approved selected origin |
| Clients, only if private HTTP approved | `JIAOYI_TRADING_PRIVATE_ORIGIN` | Exact internal HTTP origin and port pin |

Secrets must remain out of Git, reports and model prompts. No production variables are provisioned by this PR.

## 5. Tests and preserved code

Local verification includes nine focused Rust route/auth/research-guard tests, offline workspace/all-target check, offline workspace/all-target Clippy with warnings denied, and formatting. Real-router tests use a lazy local pool and prove no DB connection is acquired by anonymous private health or denials. Authorized research tests use rejected account inputs, so no broker work starts.

The full research Python suite ran 125 tests, with one existing optional skip, and passed. The full Engineering suite ran 238 tests, with one existing optional skip, and passed. Both suite counts include their skipped test. Python verification uses fake HTTP and an external-network guard, with production credentials removed. Independent review approved an unmerged implementation PR after the persisted-task compatibility issue was corrected.

Strategy 3, core strategy/session/exit code, cTrader trading authority, store settings, config, Dockerfiles, Railway configuration and workflows remain unchanged. Engineering's `PROTECTED` policy and inbound routes remain unchanged. Only the named outgoing-client migration and its tests extend into protected Engineering files.

## 6. Verified external facts and release blockers

A read-only GitHub metadata request confirmed the repository is public. Restricting Railway cannot hide financial reports already committed to this repository or revoke existing copies. No repository visibility change was made. Report storage/access needs its own owner decision; this PR includes no new account or financial snapshot.

An initial Railway metadata request received HTTP 403 with edge error 1010. Retrying with an explicit read-only API client User-Agent succeeded without replacing the credential. At 18:29:59 SGT, metadata showed one generated public domain (`jiaoyi.up.railway.app`, target port 8080), no custom domains, and one successful active deployment on the expected production-reference commit. The service-instance health and replica override fields were null; that does not override the repository's `/health` declaration or establish the effective platform routing. Private-network connectivity, other forwarding/TCP exposure, address-family compatibility and platform health routing remain unverified. Trading still binds IPv4 in the unchanged startup code; do not assume a private hostname is reachable or supports the public edge's TLS behavior.

Additional gates: Engineering's separate public listener is not secured by this Rust guard; external operator/cloud access and report delivery must be demonstrated; all generated/custom domains or forwarding routes need inventory; a future viewer must read saved reports without Trading/DB credentials or per-page live requests.

## 7. Paired rollout and rollback — not executed

| Server | Clients | Compatibility |
|---|---|---|
| Legacy | Legacy | Existing behavior, not secured |
| Legacy | Scoped | Owner/research credential checks can fail; not a valid completed rollout |
| Private | Legacy | Private diagnostics/research fail; minimal liveness alone is not readiness |
| Private | Scoped | Intended end state, proven by offline contracts; real private connectivity still pending |

Before merge/deployment approval:

1. Verify actual private origins, ports, transport/address families and platform health in an isolated nontrading harness. Never launch a second Trading actor set against the real accounts.
2. Inventory every caller and execution location, including owner/cloud tools outside Railway. Requalify the oil adapter using cached data; ensure ongoing in-memory research jobs finish before any Trading restart.
3. Prepare the exact paired Trading/Engineering/client commits and secret names. Do not assume changing clients first is seamless. Any brief monitoring gap must be explicit; never treat unavailable evidence as success.
4. Approve a controlled Trading restart because these web handlers are embedded in its binary. Verify account loops, human/S1/S3 ownership, positions/orders, opening references and exit deadlines after restart, and compare latency/deadline observations with a declared baseline. No microsecond-level performance guarantee is claimed by code review.
5. Verify explicit private profile, valid audit/research operations, denied cross-role operations and working owner report access before removing public exposure. Check every public entry point separately.
6. Roll back clients/server together while retaining the restrictive HTTP profile/network boundary. Restoring the old public image or legacy mode on an exposed domain would reopen the original data exposure. If no working restrictive rollback exists, block cutover; do not silently reopen the dashboard.

No merge, deployment, domain removal, broker request, Telegram message, production DB write or production environment change is performed by this preparation.
