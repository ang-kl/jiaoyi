# PR #249 production rollout and SQL latency follow-up

№ 18,581 - 21-11 '26 22:02 SGT | Railway `jiaoyi_prj` / `production` / `jiaoyi`

**Decision.** The approved commit is running and the application completed migration `20261121000022_hot_read_indexes.sql`. Deployment and migration work are complete. Two checks remain open: an observed HTTP 200 from `/health` and the recurring slow outcome query and supervisor warnings. The production evidence does not support closing the latency issue. [§18,581.1] [Railway deployment](https://railway.com/project/26f609a2-afc0-4e94-90a2-fe87ba40a713/service/0f67b4a0-629d-42f1-97f1-8910887896af?id=c19efbe0-df4e-4f12-8c5a-3937c12c7eaa&environmentId=f3b64526-67e3-4ea6-98af-274043a8b657)

## What shipped

PR #249 added six indexes for outcome, receipt and tick lookups. It also changed the outcome query to materialize three intermediate results, changed tick statistics to use per-account lookups, and batched tick-history inserts with PostgreSQL `UNNEST`. Its five-file diff covered three store modules, one migration, and one integration-test file. The production image also includes the earlier PR #248 because its merge commit `b19d3368` is the parent of #249. [§18,581.2] [PR #249 diff](https://github.com/ang-kl/jiaoyi/pull/249/files) · [Production commit](https://github.com/ang-kl/jiaoyi/commit/1aa864bca1528aff1eab880f2c624fddf61c7f78)

## Why the deployment appeared late

GitHub CI tests the code and builds the image. Railway creates deployments through its connection to the production branch. The first Railway settings screenshot showed the repository connected and offered **Connect Environment to Branch**. Both #248 and #249 had merged without a Railway record, and their changes matched the configured watch paths. The missing branch connection explains the absent trigger. The records do not establish when or by whom the branch was disconnected. After reconnection, the dashboard showed `main`, automatic deployments enabled, and **Wait for CI** on. Connecting the branch did not immediately produce a deployment for the already-merged SHA. [§18,581.3] [CI workflow](https://github.com/ang-kl/jiaoyi/blob/1aa864bca1528aff1eab880f2c624fddf61c7f78/.github/workflows/ci.yml) · [Railway configuration](https://github.com/ang-kl/jiaoyi/blob/1aa864bca1528aff1eab880f2c624fddf61c7f78/railway.toml)

| Time on 21 November 2026 (SGT) | Evidence |
| --- | --- |
| 14:32:29 | PR #248 merged to `main`. Its main CI passed. Production continued to show `3fca1b84`. |
| 20:18:55 | PR #249 squash-merged after its approved head `60e28b9970f71b6db24689138f6c3045f39a9b2e` passed CI. The resulting `main` SHA was `1aa864bca1528aff1eab880f2c624fddf61c7f78`. |
| 20:26:00 | Post-merge main CI completed successfully. All three jobs passed (`check`, `docker`, `engineering`). |
| 21:44–21:45 | Railway confirmed `main` connected with Wait for CI enabled. The latest production deployment was still `3fca1b84`. The owner was directed to Railway’s Command Palette (`⌘ K` on Mac) → **Deploy Latest Commit**, after GitHub confirmed `main` pointed to `1aa864bc`. |
| 21:46:36–21:48:46 | Railway created deployment `c19efbe0-df4e-4f12-8c5a-3937c12c7eaa` for `1aa864bc`. The uploaded build log shows the release image pushed under this deployment ID. Railway marked it `SUCCESS` at 21:48:33 and removed the old deployment at 21:48:46. |
| 21:58:53 | The new process logged the six-index migration statement, then `migrations applied`, then the HTTP listener on port 8080. |

[§18,581.4] [PR #248 main CI](https://github.com/ang-kl/jiaoyi/actions/runs/44876033517) · [PR #249 pre-merge CI](https://github.com/ang-kl/jiaoyi/actions/runs/44893917711) · [PR #249 main CI](https://github.com/ang-kl/jiaoyi/actions/runs/44903276489)

## What the startup and health evidence means

The uploaded deploy log ended after two lines: `Starting Container` and SQLx reporting that `_sqlx_migrations` already existed. The latter refers to the tracking table. The later Railway logs for the **same deployment** show all six `CREATE INDEX IF NOT EXISTS` statements from migration 0022, with an elapsed time of **622.19 seconds**. The application logged `migrations applied` at **21:58:53.632 SGT** after the migrator returned successfully. This confirms completion at application level. The index clauses allow existing indexes, so the log cannot identify which indexes were newly created or which ones PostgreSQL used for later queries. [§18,581.5] [Migration SQL](https://github.com/ang-kl/jiaoyi/blob/1aa864bca1528aff1eab880f2c624fddf61c7f78/migrations/20261121000022_hot_read_indexes.sql) · [Startup sequence](https://github.com/ang-kl/jiaoyi/blob/1aa864bca1528aff1eab880f2c624fddf61c7f78/crates/jiaoyi/src/main.rs#L36-L61)

The application bound port 8080 and logged `serving /health` at **21:58:53.707 SGT**, about ten minutes after Railway marked the deployment successful. Railway reported one running replica. Its deployment-scoped HTTP stream contained zero requests through **22:02:18 SGT**. The service had no public domain and the check environment could not reach its private hostname, so the HTTP response code remains unverified. [§18,581.6] [Health handler](https://github.com/ang-kl/jiaoyi/blob/1aa864bca1528aff1eab880f2c624fddf61c7f78/crates/jiaoyi/src/http_access.rs#L49-L59)

## What still needs attention

The review covered all **82 runtime records** returned for the new deployment from its creation through **22:02:18 SGT**, including just over three minutes after the listener started. The old deployment's handover warnings were counted separately. [§18,581.7]

| Finding on the new deployment | Significance |
| --- | --- |
| 16 slow SQL statements | One was the migration itself. Of the other 15, **11** were the outcome query beginning `WITH owned AS MATERIALIZED`, taking **9.82–11.46 s** each. The other four were two commits and two other queries. |
| Two Demo Strategy 3 `supervisor assessment unavailable` warnings | They occurred at **22:01:03** and **22:02:03 SGT**, each immediately after a roughly ten-second outcome query. The timing warrants checking the supervisor's deadline and the query plan. It does not establish the cause. |

**Next steps.** Obtain one timestamped HTTP response from the running service and investigate the repeated outcome query under comparable load. From an authorized client with `curl` on Railway’s production private network, the following probe prints the status code without logging response contents. For the latency follow-up, use the exact deployed `FACTS_SQL`, inspect its PostgreSQL plan and index usage with representative parameters, and compare query duration and supervisor warning frequency over matched observation windows. The current production commit can stay in place while those checks proceed. [§18,581.8] [Deployed outcome query](https://github.com/ang-kl/jiaoyi/blob/1aa864bca1528aff1eab880f2c624fddf61c7f78/crates/jiaoyi-store/src/outcomes.rs#L39)

```sh
curl --silent --show-error --max-time 5 --output /dev/null \
  --write-out '%{http_code}\n' http://jiaoyi.railway.internal:8080/health
```
