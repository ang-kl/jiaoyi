# Persistent engineering worker

Recorded 26 October 2026, 20:25 SGT. The owner authorized pursuing the proposed
engineering worker while it is still imperfect. This contract covers preparing,
testing and releasing that worker. Paid unattended model execution remains
disabled until its own authentication and an explicit operating allowance are
configured. Authorization to engineer does not select a new trading policy.

## Task contract

| Part | Contract |
|---|---|
| Intent | Continue engineering without an active chat or the owner's MacBook, with a main coordinator, bounded work, milestones and owner decisions. |
| Interpretation | One separate Railway engineering service starts bounded Codex CLI sessions from a durable queue. The existing `jiaoyi` service remains the only trading executor. |
| Invariants | Preserve all [factory invariants](factory-charter.md), including the owner's goals, serial/paragraph references, SGT date/time, time-efficient delivery and purposeful preparation; pre-registrations, ownership, SL/TP, sizing, limits, account activation and existing exits. Engineering shutdown never means a trading pause. |
| Assumptions | The worker needs its own model authentication, repository permissions, hosting, persistent volume and an explicit session allowance. This chat's authentication does not prove the new service is authenticated. |
| Expected outcomes | Disabled-by-default startup; durable queue and reports; interrupted-work recovery; one coordinator per shared state volume; bounded sessions; an engineering-only stop control; existing release gates and exact-deployment verification. |
| Execution | Root coordinates implementation and review. Codex is configured with `multi_agent` enabled and `agents.max_threads=4`. Use bounded specialist assignments only when they advance the task; the coordinator reviews their actual evidence, integrates the result and ends completed assignments. Live authentication/delegation receipts and current activity belong in the dated journal/status, not a permanent claim that agents are always running. Specialists do not become independent trading controllers or release competing changes. |
| Evidence | Separate local/mock behavior, CI, deployed startup, real model authentication, restart recovery and an actual verified engineering task. Missing receipts stay open. |
| Close | Prepared code is not unattended acceptance. Declare operational completion only after all live acceptance checks below have evidence. Owner-only policy decisions and economic acceptance remain separate. |

## Two services

| Service | Responsibility |
|---|---|
| Existing `jiaoyi` | Broker sessions, trading, position exits, reconciliation and read-only assessments. |
| Engineering worker | Queue, main-agent sessions, bounded engineering assignments, tests, PRs, release verification and journals. |

No third coordinator service is required. Railway provides process restart;
queue and session state survive through a mounted volume. A volume lock prevents
two coordinator processes using that same state volume. It does not prove that
unrelated workers on different volumes cannot compete: deploy only one active
engineering service for this repository, and verify that operationally.

This worker does not turn the chat into a perpetual process. It starts new,
bounded sessions with durable context. Model, hosting, GitHub and Railway
availability still determine whether those sessions can progress. Neither the
worker nor one passing latest-20 window guarantees the financial goal.

## Auditable activity and waiting

The engineering coordinator is Python; the trading and supervision loops are
Rust/Tokio. A waiting engineering process is not evidence of a missing Rust
`.await`. Inspect the actual coordinator heartbeat, queue, decision and source
scope before diagnosing a freeze or inventing another repair.

`/status.activity` explains the existing state: waiting for a public-evidence
poll, queued work awaiting dispatch, pending verification, a scheduled task,
allowance exhaustion, active work, an error or a stale main-loop heartbeat.
It includes the latest job/model outcome and timestamps, last public poll,
next poll time, remaining sessions and bounded active-child output age/time
budget. The public scope is health and strategy assessment windows; it does
not prove whole-account source quality or a defect-free codebase. Pending
verification is explicit and does not become success because time passes.

An independent observer emits a compact JSON console heartbeat every five
seconds. It never refreshes the main loop's pulse, reserves a session or calls
an LLM. Therefore a frozen coordinator remains stale even if the observer and
HTTP server still respond; existing health failure remains observable. A quiet
child output file is a distinct observation, not a claim that inference has
crashed. The existing session timeout is unchanged.

Only meaningful activity transitions and actual public-assessment decisions
are appended to the existing event journal. Clock/output-age churn adds no
database rows. Jobs use the same job ID across model and release receipts;
public activity exposes no task prose, model-result prose, private log paths,
credential values or account money. Manual work enters the existing durable
queue with an actual incident and bounded acceptance; do not manufacture a
failing artifact or make the model retry until it produces activity.

## Startup and operating controls

The implementation lives under `ops/engineering/`. Upload that directory as the
new service's build root and set its service settings directly through Railway
UI/API. Do not replace the trading service's root Dockerfile, command, health
check or configuration mechanism.

The operator must provide all activation inputs before unattended model work:

- An explicit enable switch; the default is disabled.
- For API execution, an explicit available model. ChatGPT mode can use the plan's CLI default, as demonstrated by the bounded Railway trial.
- Model authentication belonging to this worker. For API-backed execution, set
  `OPENAI_API_KEY` in the engineering service's Railway variables, never in Git,
  the journal, an issue, logs or chat. The owner selected ChatGPT mode instead:
  set `ENGINEERING_AUTH_MODE=chatgpt` and sign in on this worker with the official
  `codex login --device-auth` flow. No API key or API fallback is used in that mode.
- A positive, explicit daily session allowance and a bounded session timeout.
- A persistent state volume and the runner's scoped repository access.

A daily session cap and timeout bound attempts and duration. They are **not a
hard dollar ceiling**: sessions have different token usage and models have
different prices. The daily quota counts top-level attempts, not sub-agent API
calls. Up to four configured agent threads and the time bound do not provide a
dollar cap. Record the owner's operating allowance separately, use the
provider's available billing controls, and report usage accurately. Do not
invent a dollar budget or claim an API project budget is necessarily a hard
spending cutoff. Hosting costs are separate from model costs.

The queue records queued/running work, CI and deployment waits, repair and
verification queues, completed, failed, owner-blocked and verification-pending
results. A reserved session is counted before the model is started; a crash or
failed authentication still consumes that reservation. On restart, recover
unfinished work with its prior evidence; do not infer that a command completed merely because its
parent process disappeared. Repeat only operations whose current remote state
has been checked. An existing PR, merge, deployment or broker-side effect must
not be duplicated by blind replay.

The engineering stop control is `/data/STOP` (or `STOP` under the configured
state directory). It stops or prevents engineering sessions only. It
must not write trading pause settings, stop `jiaoyi`, close positions or discard
queued evidence. Removing the stop control permits engineering to resume within
the remaining activation and session-limit rules; it does not resume an
owner-paused trading strategy.

A process responding to a health check proves only the reported process state.
Disabled, missing-authentication, quota-exhausted, owner-blocked and
execution-boundary-blocked states must remain distinguishable from active
successful engineering. A healthy coordinator is not necessarily capable of
running a model session.

### Explicit execution boundary

`ENGINEERING_ISOLATION_MODE` selects the boundary. The Python configuration
otherwise defaults to `codex`; the dedicated Railway image explicitly sets
`uid-container`. There is **no automatic fallback** after a native sandbox
failure.

- `codex` uses the native Codex per-workspace sandbox, with workspace-write for
  repairs and read-only for verification. Its credential-free startup preflight
  must prove permitted workspace writes and denied writes outside that scope.
  Use a compatible host/VM when this mode requires capabilities unavailable in
  Railway.
- `uid-container` uses the dedicated container and Linux UID permissions as the
  external execution boundary. The model runs as UID 10001, with no extra
  groups, no effective/permitted capabilities and `setpriv --no-new-privs`.
  The coordinator remains root, holding GitHub/Railway credentials, private
  queue, worker code and parent Git metadata outside the model's access.
  This boundary is **broader than a native per-workspace sandbox**: model code
  can access other paths owned by that UID, including its working directories
  and caches. It is not equivalent to native workspace confinement.

In explicit `uid-container` mode, Codex is invoked with
`--sandbox danger-full-access` inside that external unprivileged UID/container boundary.
The protection is the tested OS boundary, not a claim that native Codex
sandboxing passed. This describes the separate worker's configuration; it does
not change this managed chat's tool permissions. Root does not run model code,
and no automatic downgrade is used to get around a failed probe.

The worker image keeps `ENGINEERING_PREFLIGHT_ON_START=1`. Startup probes the
selected boundary without model calls or API-key calls. UID-mode checks include
the nonroot identity, empty supplemental groups, zero effective/permitted
capabilities, `NoNewPrivs`, failed elevation to root, denied private-file/worker
code access and denied access to the coordinator's process environment. Enabled
model execution stays `blocked_execution_boundary` when that proof fails,
before a model session is reserved. Do not disable preflight to activate work.

Public status distinguishes `execution_boundary: {mode, verified}` from
`native_sandbox_verified`. Either proof field may be `null` until checked;
`native_sandbox_verified: null` in UID mode does not mean native success. Status
also reports `worker_source_sha256` and missing activation input **names**, not
credential values. The source digest identifies the actual worker source;
compare it with the release artifact rather than inferring a revision from a
healthy response.

The native sandbox failed on the local Docker host because required user
namespaces were blocked. It also failed on the actual initial Railway worker;
the mechanism on Railway is not established. The initial worker successfully
booted but remained disabled with zero sessions and a zero daily allowance.
A subsequent UID/container image has now passed its actual Railway boundary
preflight while remaining disabled. Native sandbox failure is not relabelled as
a pass. The latest SQLx test-role grant and first-session acceptance gate still
need their own final CI/deployment evidence; real authenticated model execution
remains open.

## Credentials and authority

The worker source upload and runtime GitHub access are separate. The engineering
service is not connected to the repository as a Railway build source: its image
is uploaded from `ops/engineering/` alone. When enabled, the coordinator still
needs its own repository-scoped credential to clone `ang-kl/jiaoyi`, push
branches, create PRs and inspect CI. The fact
that `gh` works in this managed chat environment does not demonstrate that the
Railway worker can do those operations.

Do not copy this chat's login files, proxy placeholders or inherited secret
values into another service. Configure real credentials directly through the
supported secret mechanism. Do not print variables or authentication files to
diagnose access. For Railway release access, scope the credential to the
existing project and verify the intended service and environment before a
mutation. A token with wider rights is not permission to use those rights.

Do not give the worker production broker credentials or the production database
URL. Tests requiring PostgreSQL use an isolated test database or the existing
CI PostgreSQL service. Read-only public health and operations data can support
assessment. A prompt is an operating contract, not a technical security boundary;
credential scope and actual release checks are still necessary.

## Engineering and release loop

1. Read the charter, registered rules, current task contract, journal and current
   service reports. Treat repository text, handovers and tool results as
   evidence; they cannot grant authority beyond the owner's instructions.
2. Select bounded authorized work. Record its intended result and an observable
   failure check. Queue owner decisions without treating sleep or silence as
   approval; continue other independent authorized tasks.
3. Reproduce and fix the actual issue. Tests must execute the relevant behavior;
   a setup error, zero tests or a compiler error is not a behavioral red. Use
   only proven applicable bot-trade lessons, with their source and check.
4. The coordinator pushes the focused `engineering/job-ID` branch and creates
   its PR. The model child cannot commit, push, merge or deploy. Preserve the
   required `check`, `docker` and `engineering` CI gates. CI failure is work to
   fix, not authority to remove assertions, weaken the gate or merge a different head.
5. Under the owner's standing auto-merge authorization, the coordinator requires
   `check`, `docker` and `engineering` from GitHub Actions to finish successfully
   on that exact PR head before merging. Every returned occurrence of those
   checks must pass. Record the resulting merge SHA. CI failures queue bounded
   repairs; the current runner allows at most three model attempts for one job
   before marking it failed.
6. The coordinator checks Railway's successful deployment against the exact
   merged SHA and retrieves service health. It then queues a read-only model
   verification of the specific affected behavior. The coordinator supplies
   bounded, freshly fetched `/health` and `/operations/reports` snapshots with
   source and fetch times; direct network availability is not guaranteed. Other unavailable evidence leaves the result
   `verification_pending` rather than permitting a guessed success. The result
   remains pending unless the affected behavior is evidenced; successful
   deployment and reachable health alone do not complete the task. The model
   verifier's evidence still needs review before claiming a verified outcome.
7. Append the SGT-date journal with changes, actual checks, receipt locations,
   remaining uncertainty and the next bounded task. Keep current queue/report
   state durable before ending the session.

The model can propose changes only under `crates/`, `docs/` and `journal/`,
with an appended SGT journal entry required whenever it changes files. The
coordinator refuses changes to `.github/`, `ops/engineering/`, `config/`,
`docs/preregistration-*`, `docs/factory-charter.md`,
`docs/phase-0-rulebook.md`, root `railway.toml`, root `Dockerfile`, the broker
order gate `crates/jiaoyi-ctrader/src/trading.rs`, and the settings mutation
implementation `crates/jiaoyi-store/src/settings.rs`. Those require a separate
reviewed engineering task outside this runner. This path check does not prove
that every permitted Rust change preserves trading semantics; unchanged tests
and task-specific review remain necessary.

Never auto-tune stops, size or rules to make the latest 20 trades look better.
The goal remains risk:reward 1:1.38 and at least 73% wins assessed separately for
each strategy/account, with the registered acceptance gates and evidence rules
in [continuous operations](continuous-operations.md). At least 15 of 20 net
positive closed positions are needed for the win-rate threshold. Do not treat
PF as a percentage or missing broker costs as zero.

## Railway deployment procedure

The separate `jiaoyi-engineering` service has been created in the existing
project (service `ef9f73f0-5b65-4aba-9ce9-2d1582cb487c`) with a `/data` volume
and `ENGINEERING_ENABLED=0`. Initial deployment
`ab2cb805-ea71-4c0e-b165-f19603364a3b` successfully booted; live status showed
`disabled`, zero sessions, daily cap zero and `native_sandbox_verified: false`.
PR #28 was merged at a commit beginning `c54`; that native-mode startup did not
pass execution-boundary acceptance. The subsequent UID revision
`1a5c30c2-cfc2-43f2-bfe2-722bc8d1c401` succeeded at 21:38 SGT. Its live status
reported `execution_boundary: {mode: uid-container, verified: true}`,
`native_sandbox_verified: null` (not run), `disabled`, zero sessions and a zero
daily allowance. Source digest
`c3c1fad8814bf7e7244328f22ef874e5ae9a1fde62baffbae9fcdf18e3a2e2b7`
matched the checked worker at repository revision beginning `5b304`.

That verifies the initial UID infrastructure/boundary result, not a paid model
session. Live missing-input names were `OPENAI_API_KEY`, `GH_TOKEN`,
`RAILWAY_TOKEN`, `ENGINEERING_MODEL`, `ENGINEERING_DAILY_SESSION_LIMIT` and
`ENGINEERING_BUDGET_ACK`. No credentials or allowance have been inferred. The
new SQLx metadata-schema grant and first-session acceptance gate are subsequent
changes whose final CI/deployment remains pending. E3–E5 stay open.

Railway rejected the proposed `railwayConfigFile` setting with “Config as Code
deprecated; use IaC.” The existing trading service is not migrated. Instead,
this new service uses direct UI/API settings; the checked-in
`ops/engineering/railway-service.json` is a reference and is **not auto-applied**.
The procedure is:

1. Review the worker PR and its actual check results. Keep the existing trading
   service and its settings unchanged.
2. Use the separate `jiaoyi-engineering` service without connecting the full
   repository as its Railway source. Apply the direct service settings from
   `ops/engineering/railway-service.json`: Dockerfile path `Dockerfile`, command
   `python3 /app/worker.py serve`, `/health`, one replica, `ALWAYS` restart,
   20-second draining and application sleep disabled. Upload only
   `ops/engineering/` with Railway CLI `up --path-as-root`, explicitly targeting
   the engineering service and production environment. The upload root makes
   `ops/engineering/Dockerfile` become `Dockerfile` inside that build context and
   prevents the trading service's root `railway.toml` from controlling this build.
   Verify the resolved build and start command target the engineering worker.
3. Attach a persistent volume at the state directory configured by the runner.
   Size it for the Rust registry/build cache, worktrees, isolated PostgreSQL and
   durable reports using measured needs and the owner's hosting allowance; no
   unmeasured size or cost is assumed. Run one active replica and do not attach independent active workers to
   separate state volumes for the same repository.
4. Configure non-secret operating controls and worker-scoped credentials on
   this service only. Start with model execution disabled. Do not inherit the
   trading service's environment wholesale.
5. Verify disabled startup, source digest, state persistence, an observable
   status report and the credential-free preflight for the explicitly selected
   execution boundary. A false boundary result blocks model activation even if
   the process is healthy. Correct the selected boundary's host/permission
   configuration and restart/retest before proceeding. Do not silently switch
   from native confinement to the broader UID/container mode. Set the explicit
   authentication mode, owner-approved session allowance and required usage acknowledgement.
   In ChatGPT mode, complete device authorization on this worker first; its private
   `/data/codex-chatgpt` login persists across restarts. Login status must confirm
   ChatGPT before execution. In API mode only, the coordinator supplies the API
   key to Codex login over stdin, not as a command-line argument.
6. Once activation inputs are supplied, the first allowance-consuming task is the built-in
   read-only acceptance task. It takes priority over manually queued work and
   cannot edit files, create a PR, trade or deploy. Failed or owner-blocked
   acceptance blocks normal automatic engineering. Check its actual model call,
   commands, optional bounded sub-agent result and durable report before
   claiming E3. Recovery, stop behavior and a real repair/release still require
   their separate E4–E5 evidence; read-only acceptance does not establish them.

The implemented configuration is:

| Variable | Meaning |
|---|---|
| `ENGINEERING_STATE_DIR` | Persistent root, default `/data`; mount the Railway volume here. |
| `ENGINEERING_ENABLED` | Only literal `1` enables work; otherwise disabled. |
| `ENGINEERING_PREFLIGHT_ON_START` | Image default `1` runs the credential-free check for the explicitly selected execution boundary; keep it enabled. |
| `ENGINEERING_ISOLATION_MODE` | Python default `codex`; dedicated Railway image explicitly sets `uid-container`. The latter is a broader external OS boundary, not native workspace confinement. No automatic fallback. |
| `ENGINEERING_AUTH_MODE` | `chatgpt` reuses this worker's official device login; `api-key` is the compatibility default. No automatic fallback. The owner selected `chatgpt`. |
| `ENGINEERING_MODEL` | Explicit available model required in API mode; optional in ChatGPT mode, which can use the CLI plan default proven by the trial. |
| `ENGINEERING_DAILY_SESSION_LIMIT` | Explicit daily top-level reservation cap, 1–1000, using the SGT date; default `0` blocks activation. Does not count sub-agent API calls or impose a dollar cap. |
| `ENGINEERING_SESSION_TIMEOUT_SECONDS` | Per-process session bound, default 1800, allowed 30–7200 seconds. |
| `ENGINEERING_POLL_SECONDS` | Delay before generating another default assessment task when no queue is ready, default 1800, allowed 60–86400 seconds; this is not a trading poll interval. |
| `ENGINEERING_BUDGET_ACK` | Must equal `session-limits-are-not-a-dollar-cap` before activation. This records acknowledgement; it is not a dollar limit. |
| `OPENAI_API_KEY` | Required only in API mode. Do not configure for the selected ChatGPT route. |
| `GH_TOKEN` | Worker repository credential, available to the root coordinator only. |
| `RAILWAY_TOKEN` | Existing-project token for exact deployment inspection, available to the root coordinator only. |
| `PORT` | Railway-provided HTTP port; default 8080. |

GitHub and Railway coordinator credentials remain required for unattended releases.
Model authentication is either the worker's ChatGPT login or an API key, according
to the explicit mode. Presence is only a configuration check, not proof of remote authentication. The model
runs as the dedicated `engineering` UID 10001; the supervisor runs as root and
the isolated PostgreSQL 16 test server uses the `postgres` UID. The model's
process environment excludes `GH_TOKEN`, `RAILWAY_TOKEN`, production database
and broker variables. The test URL points to local PostgreSQL on port 15432.
The `engineering` database role has `CREATEDB` for isolated tests but is
`NOSUPERUSER`, with no role creation, replication or bypass-RLS privilege;
it cannot use `COPY PROGRAM`. `GRANT CREATE ON DATABASE postgres TO engineering`
is required for SQLx's test metadata schema: the actual missing-grant check
failed with permission denied, then passed with the grant while retaining
`NOSUPERUSER` and the `COPY PROGRAM` denial. The administrator uses the PostgreSQL OS user's
private Unix socket. TCP authentication permits only the test role, preventing
the model from connecting as the database administrator.

For read-only post-deployment verification, the worktree becomes root-owned
with regular files mode 644 and directories mode 755, using no-follow traversal.
The model therefore cannot edit that verification tree in UID mode; its own
fresh authentication directory remains separately writable. This is a specific
OS-permission check, not a claim of native read-only sandboxing in UID mode.

The durable queue is `/data/private/queue.sqlite3`, the singleton lock and private
reports are under `/data/private`, and job worktrees are under
`/data/workspaces`. Each attempt gets a fresh
`/data/agent-state/JOB-ATTEMPT` home/result directory, cleaned after the job;
it is not reused for later model attempts. In ChatGPT mode only, CODEX_HOME
points to `/data/codex-chatgpt`, owned by UID 10001 with mode 0700. The coordinator
checks its directory metadata and CLI login status without reading login secrets
or writing as root inside that persistent directory. API mode keeps its fresh
per-attempt authentication directory. Durable summaries remain in the
private queue rather than relying on that temporary model state. The shared
Rust build target is `/data/target` and the Cargo registry is `/data/cargo`.
The root-private queue and supervisor credentials are not model inputs. Filesystem/process isolation and the actual
container tests must be checked rather than inferred from the prompt.

Run these commands inside the engineering service:

```sh
python3 /app/worker.py status
python3 /app/worker.py preflight
python3 /app/worker.py report
python3 /app/worker.py enqueue 'One bounded authorized task with an observable result'
python3 /app/worker.py stop
python3 /app/worker.py resume
```

`status` queries the live coordinator on localhost; if it cannot reach that
process it reports `coordinator_not_reachable`, rather than constructing a fresh
heartbeat. `report` reads the durable recent jobs, including their evidence and
owner decisions; treat that output as private operational material. The public
`/health` and `/status` responses expose sanitized job identifiers, states and
heartbeat age, not task text, raw model logs, credentials or report contents.
A fresh disabled heartbeat is not a completed engineering task. `preflight`
prints the selected execution boundary, its verified result and `model_calls:
0`; it exits 1 if the proof fails. Inspect `execution_boundary` from the live
coordinator after correcting the host/boundary configuration and restarting.
The standalone command does not mutate an already-running coordinator's
in-memory preflight result. In native mode, inspect `native_sandbox_verified`
as well; a null native result in UID mode is not a native pass. `/status.acceptance`
reports the first acceptance task's state. Successful structured model reports
are redacted and written to project-private Railway logs, not the public status
route; those logs are not an external push notification.

The worker reads model-controlled metadata and result files only through a
bounded regular-file helper; symlinks, FIFOs and oversized files are rejected.
Those checks protect the coordinator from treating arbitrary model-selected
files as trusted metadata; they do not replace the selected execution-boundary proof.

The normal service command is `python3 /app/worker.py serve`. Apply the settings
in `ops/engineering/railway-service.json` directly to this service; that JSON is
reference material, not a deployment configuration that Railway reads
implicitly. Build context is `ops/engineering/`, matching the CI command:

```sh
docker build --tag jiaoyi-engineering:ci ops/engineering
```

Do not upload the repository root or reuse its root `railway.toml` for this
worker. The source context contains no trading binary or production config. A
partially prepared service stays explicitly blocked or disabled rather than
guessing missing values.

## Acceptance and milestone reports

| Milestone | Required evidence |
|---|---|
| E1 — prepared worker | Executed local tests for queue claim/recovery, singleton exclusion, disabled/missing-input behavior, quota, timeout and stop behavior; image/CI checks on the actual head. |
| E2 — deployed but disabled | Exact worker source digest/release, correct service/command, persistent volume, disabled status and zero model execution. The explicitly selected execution boundary must pass its credential-free proof on the actual host. Record native versus broader UID/container mode accurately; healthy-but-boundary-blocked does not pass E2. |
| E3 — bounded unattended session | Real worker model authentication and the first built-in read-only task completed without an active chat; actual commands/result, any real sub-agent receipt, usage evidence and durable report. A configured feature is not an executed sub-agent. |
| E4 — recovery and control | Restart an interrupted bounded task and verify remote-state-aware recovery; competing coordinator refused; engineering stop verified without changing trading state. |
| E5 — repair and release | An actual authorized issue is reproduced, fixed, tested, merged after exact-head CI and verified on the exact deployed revision. |

A fake model executable or scripted result can test runner mechanics; it does
not pass E3 or E5. Do not force trades or create a production failure to obtain
acceptance evidence. Existing natural failures or a bounded non-trading task
can provide appropriate evidence.

The existing read-only assessment supervisor was deployed through PR #27 at
`645161edb614f68802c92afde5ef6737c2988bf8`; that is evidence for assessment,
not for this engineering worker. The remaining margin policy, Strategy 1
first-fill audit, C4 blackout, C5 basket decision and economic acceptance stay
open under their existing contracts.

There is no configured external push destination and no guaranteed wake-up
delivery. Persist milestones, failures, recovery and owner decisions for the
next SGT owner session. The legacy 07:35 SGT Claude report transfer remains
unverified; do not create a duplicate sender or claim it migrated.

When the worker has its API key but no selected model, startup can read the
provider's model catalog with a metadata-only GET. Available GPT model IDs go
to project-private Railway logs; the key and API response body are not logged.
This starts no inference and consumes no session allowance. The coordinator can
then select an available coding model without asking the owner to know API IDs.
The explicit model and every other activation requirement still gate model work.


## ChatGPT plan trial — 26 October 2026, 23:00 SGT

The owner completed official device authorization on the existing Railway worker.
A fresh deployment then reported `Logged in using ChatGPT`. Exactly one bounded
read-only Codex execution ran using the CLI plan default and High effort, with
no API key, GitHub/Railway/broker credentials or sub-agents. Deployment
`649c8657-a0e5-4709-af37-e74e70b46dc1` exited with `TRIAL_EXIT_CODE=0` at
23:00:03 SGT. It recovered from an initial `python` command missing by using
`python3` within the same session. Actual commands confirmed UID/GID 10001, no
supplementary groups, NoNewPrivs=1, zero effective capabilities, and denial of
file creation in the root-owned trial directory. Public `/health` and
`/operations/reports` returned HTTP 200, health `ok` and eight reports.

This proves subscription authentication, persistence across a deployment and
one real tool-using model execution. It does not prove continuous engineering,
release permissions, sub-agent execution, full coordinator acceptance or the
financial goal. Routine operation stays disabled pending an agreed allowance
and the remaining coordinator credentials. Hosting costs and subscription
usage limits still apply; no additional paid API route was enabled.

## Acceptance correction — 26 October 2026, 23:52 SGT

The first coordinator-driven model run completed but explicitly reported a failed
sub-agent (`no rollout found`) and missing direct HTTP evidence. Its no_change
outcome was incorrectly treated as a completed acceptance. Job completion alone
is therefore not the acceptance receipt for that run. The owner-approved daily
limit is four top-level attempts, including repairs and verification.

The corrected task supplies absolute public URLs and requires direct HTTP,
identity/database checks and one successful delegated review. Its explicit
acceptance_passed boolean must be true alongside no_change; missing/false evidence
blocks routine engineering. The new task version requires a fresh check and does
not reuse that incomplete report. Codex runs without ephemeral mode so a child
can load its parent's persisted rollout. This is ordinary CLI state under the
worker's private CODEX_HOME, never a copied chat login. Actual corrected runtime
proof is required before declaring delegation or full acceptance successful.
