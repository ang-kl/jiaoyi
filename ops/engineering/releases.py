"""Component-specific release evidence, using the coordinator's existing queue.

No orders, deployment commands, model calls or new database tables live here.
"""
import datetime as dt
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import time
import tomllib
import urllib.request

WAIT_SECONDS = 1800
POLL_SECONDS = 30
REQUIRED_CHECKS = {"check", "docker", "engineering"}
SHA = re.compile(r"[0-9a-f]{40}\Z")


class EvidenceError(RuntimeError):
    pass


def clock(value):
    if not isinstance(value, str):
        raise EvidenceError("release_time_unavailable")
    try:
        parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise EvidenceError("release_time_unavailable") from error
    if parsed.tzinfo is None:
        raise EvidenceError("release_time_unavailable")
    return parsed.astimezone(dt.timezone.utc)


def patterns(value):
    # Match the positive, root-anchored subset used by the actual service.
    if not isinstance(value, list) or not value or len(value) > 100:
        raise EvidenceError("watch_patterns_unavailable")
    for item in value:
        if (not isinstance(item, str) or not item.startswith("/") or "//" in item
                or any(p in ("", ".", "..") for p in item[1:].split("/"))
                or re.search(r"[^A-Za-z0-9_./*?-]", item)
                or "**" in item.removesuffix("/**")):
            raise EvidenceError("watch_patterns_unsupported")
    return sorted(set(value))


def watched(path, rules):
    name = "/" + path
    for rule in rules:
        if rule.endswith("/**") and name.startswith(rule[:-2]):
            return True
        parts, globs = name.split("/"), rule.split("/")
        if len(parts) == len(globs) and all(fnmatch.fnmatchcase(p, g) for p, g in zip(parts, globs)):
            return True
    return False


def checks_passed(data):
    rows = data.get("check_runs") if isinstance(data, dict) else None
    if not isinstance(rows, list):
        return False
    relevant = [r for r in rows if isinstance(r, dict) and r.get("name") in REQUIRED_CHECKS]
    return (REQUIRED_CHECKS <= {r["name"] for r in relevant} and all(
        r.get("status") == "completed" and r.get("conclusion") == "success"
        and (r.get("app") or {}).get("slug") == "github-actions" for r in relevant))


class Releases:
    def __init__(self, coordinator, repo, service, environment, protected):
        self.c, self.store = coordinator, coordinator.store
        self.repo_name, self.service, self.environment = repo, service, environment
        self.protected = protected
        self.next_poll = {}
        self.cursor = 0
        self.verified = set()

    def saved(self, job, kind):
        with self.store.connect() as db:
            row = db.execute("SELECT detail FROM events WHERE job=? AND kind=? ORDER BY id DESC LIMIT 1",
                             (job, kind)).fetchone()
        try:
            value = json.loads(row[0]) if row else None
            return value if isinstance(value, dict) else None
        except (ValueError, TypeError):
            return None

    def record(self, job, kind, detail, state=None):
        """One material transition, atomically with its job state; no poll rows."""
        encoded = json.dumps(detail, sort_keys=True, separators=(",", ":"))
        with self.store.connect() as db:
            db.execute("BEGIN IMMEDIATE")
            prior = db.execute("SELECT detail FROM events WHERE job=? AND kind=? ORDER BY id DESC LIMIT 1",
                               (job["id"], kind)).fetchone()
            row = db.execute("SELECT state FROM jobs WHERE id=?", (job["id"],)).fetchone()
            changed = prior is None or prior[0] != encoded
            if changed:
                db.execute("INSERT INTO events(at,job,kind,detail) VALUES(?,?,?,?)",
                           (dt.datetime.now(dt.timezone.utc).isoformat(), job["id"], kind, encoded))
            if state and (changed or row[0] != state):
                db.execute("UPDATE jobs SET state=?,updated=?,result=? WHERE id=?",
                           (state, dt.datetime.now(dt.timezone.utc).isoformat(), encoded, job["id"]))

    def handoff(self, job, reason, **evidence):
        self.verified.discard(job["id"])
        state = "verification_pending" if job.get("merged") else "owner_blocked"
        self.record(job, "release_disposition", {"outcome": "verification_pending" if job.get("merged") else "owner_blocked",
            "release_reason": reason, "owner_action_required": False, "acceptance_passed": False,
            "summary": "Release evidence retained; resolve " + reason + " before completion.",
            "tests": "No new model session or deployment was started.", "owner_decisions": "none",
            "head": job.get("head"), "merged": job.get("merged"), **evidence}, state)

    def api(self, query, variables=None):
        request = urllib.request.Request("https://backboard.railway.com/graphql/v2",
            data=json.dumps({"query": query, "variables": variables or {}}).encode(),
            headers={"Content-Type": "application/json", "User-Agent": "jiaoyi-maintenance/0.1",
                     "Project-Access-Token": self.c.config.env["RAILWAY_TOKEN"]})
        with urllib.request.urlopen(request, timeout=20) as response:
            body = response.read(2 * 1024 * 1024 + 1)
        if len(body) > 2 * 1024 * 1024:
            raise EvidenceError("deployment_source_unavailable")
        reply = json.loads(body)
        if reply.get("errors") or not isinstance(reply.get("data"), dict):
            raise EvidenceError("deployment_source_unavailable")
        self.c.pulse()
        return reply["data"]

    def effective_patterns(self):
        data = self.api('query($s:String!,$e:String!){serviceInstance(serviceId:$s,environmentId:$e){watchPatterns}}',
                        {"s": self.service, "e": self.environment})
        return patterns(data["serviceInstance"]["watchPatterns"])

    def repository(self, *commits):
        repo = self.c.config.root / "repository"
        # Never use prepare(): reconciliation must not initialise a test DB or build.
        for path in (repo, repo / ".git"):
            if path.is_symlink() or not path.is_dir() or path.stat().st_uid != os.geteuid():
                raise EvidenceError("trusted_repository_unavailable")
        for commit in commits:
            if not isinstance(commit, str) or not SHA.fullmatch(commit):
                raise EvidenceError("release_identity_unavailable")
            try:
                self.c.git("cat-file", "-e", commit + "^{commit}", cwd=repo)
            except RuntimeError:
                # Fetch immutable objects from the fixed repository, not model-owned remotes.
                self.c.git("fetch", "--no-tags", "--no-recurse-submodules",
                           "https://github.com/" + self.repo_name + ".git", commit, cwd=repo, timeout=20)
                self.c.git("cat-file", "-e", commit + "^{commit}", cwd=repo)
        return repo

    def blob(self, repo, commit, path):
        return self.c.git("show", commit + ":" + path, cwd=repo, raw=True)

    def classify(self, job, base, head):
        repo = self.repository(base, head)
        wire = self.c.git("diff", "--name-status", "-z", "--no-renames", "--no-ext-diff",
                          "--no-textconv", base, head, "--", cwd=repo, raw=True)
        fields = wire.split(b"\0")
        if not wire or fields[-1] != b"" or (len(fields) - 1) % 2:
            raise EvidenceError("complete_diff_unavailable")
        changes = {}
        for index in range(0, len(fields) - 1, 2):
            status, path = fields[index].decode("ascii"), fields[index + 1].decode("utf-8")
            if (status not in ("A", "M", "D", "T") or not path or path.startswith("/")
                    or any(part in ("", ".", "..") for part in path.split("/")) or path in changes):
                raise EvidenceError("complete_diff_unavailable")
            if (any(path == p or path.startswith(p) for p in self.protected)
                    or not path.startswith(("crates/", "docs/", "journal/"))):
                raise EvidenceError("release_scope_protected")
            for commit in (base, head):
                entry = self.c.git("ls-tree", "-z", commit, "--", path, cwd=repo, raw=True)
                if entry and (entry.count(b"\0") != 1 or entry.split(b" ", 1)[0] not in (b"100644", b"100755")
                              or entry.split(b"\t", 1)[-1][:-1] != path.encode()):
                    raise EvidenceError("release_scope_protected")
            if path.startswith("journal/") and (status == "D" or (status != "A" and not
                    self.blob(repo, head, path).startswith(self.blob(repo, base, path)))):
                raise EvidenceError("journal_not_append_only")
            changes[path] = None if status == "D" else hashlib.sha256(self.blob(repo, head, path)).hexdigest()
        if not any(path.startswith("journal/") for path in changes):
            raise EvidenceError("release_journal_missing")
        before = patterns(tomllib.loads(self.blob(repo, base, "railway.toml").decode())["build"]["watchPatterns"])
        after = patterns(tomllib.loads(self.blob(repo, head, "railway.toml").decode())["build"]["watchPatterns"])
        effective = self.effective_patterns()
        if before != after or after != effective:
            raise EvidenceError("watch_patterns_mismatch")
        encoded = json.dumps(changes, sort_keys=True, separators=(",", ":"))
        return {"schema_version": 1, "scope": "trading_runtime" if any(watched(p, after) for p in changes) else "repository_only",
                "base_sha": base, "tested_head": job["head"], "changed_path_count": len(changes),
                "changed_paths_digest": hashlib.sha256(encoded.encode()).hexdigest(),
                "watch_patterns_digest": hashlib.sha256(json.dumps(after).encode()).hexdigest()}, changes

    def before_merge(self, job, base):
        try:
            plan, _ = self.classify(job, base, job["head"])
            self.record(job, "release_scope_premerge", plan)
            return True
        except Exception as error:
            self.handoff(job, str(error) if isinstance(error, EvidenceError) else "release_source_unavailable")
            return False

    def plan(self, job):
        saved = self.saved(job["id"], "release_scope")
        if saved and saved.get("tested_head") == job.get("head") and saved.get("merge_sha") == job.get("merged"):
            return saved
        pr = json.loads(self.c.gh("api", f"repos/{self.repo_name}/pulls/{job['pr']}"))
        if (pr.get("merged") is not True or pr.get("merge_commit_sha") != job.get("merged")
                or (pr.get("base") or {}).get("ref") != "main"
                or (pr.get("head") or {}).get("sha") != job.get("head")
                or (pr.get("head") or {}).get("ref") != "engineering/job-" + str(job["id"])
                or ((pr.get("head") or {}).get("repo") or {}).get("full_name") != self.repo_name):
            raise EvidenceError("release_identity_changed")
        merged_at = clock(pr.get("merged_at"))
        if merged_at > dt.datetime.now(dt.timezone.utc):
            raise EvidenceError("release_time_unavailable")
        repo = self.repository(job["merged"], job["head"])
        parents = self.c.git("rev-list", "--parents", "-n", "1", job["merged"], cwd=repo).split()
        if len(parents) != 2 or parents[0] != job["merged"]:
            raise EvidenceError("merge_shape_unverified")
        if self.c.git("rev-parse", job["merged"] + "^{tree}", cwd=repo) != self.c.git("rev-parse", job["head"] + "^{tree}", cwd=repo):
            raise EvidenceError("merge_tree_mismatch")
        checks = json.loads(self.c.gh("api", f"repos/{self.repo_name}/commits/{job['head']}/check-runs?per_page=100"))
        if not checks_passed(checks):
            raise EvidenceError("exact_head_ci_unverified")
        plan, _ = self.classify(job, parents[1], job["merged"])
        plan.update(merge_sha=job["merged"], merged_at=merged_at.isoformat(),
                    wait_deadline_at=(merged_at + dt.timedelta(seconds=WAIT_SECONDS)).isoformat())
        self.record(job, "release_scope", plan)
        return plan

    def repository_verification(self, job, plan):
        # Only the private coordinator/operator can record this event. Model
        # results and task prose are never an acceptance declaration.
        receipt = self.saved(job["id"], "repository_acceptance") or {}
        fresh, changes = self.classify(job, plan["base_sha"], job["merged"])
        if fresh["scope"] != "repository_only" or fresh["changed_paths_digest"] != plan["changed_paths_digest"]:
            raise EvidenceError("release_scope_changed")
        if (receipt.get("head") != job["head"] or receipt.get("source") not in ("owner_approved_artifact", "independent_review")
                or receipt.get("criteria") != "exact_artifact_content" or not isinstance(receipt.get("reference"), str)
                or not receipt["reference"].strip() or receipt.get("files") != changes):
            self.handoff(job, "repository_acceptance_missing")
            return False
        self.record(job, "release_disposition", {"outcome": "verified", "acceptance_passed": False,
            "release_reason": "repository_artifacts_verified", "verification_scope": "repository_only",
            "head": job["head"], "merged": job["merged"], "owner_action_required": False,
            "summary": "Exact approved repository artifacts verified; trading runtime and financial edge are not certified.",
            "tests": "Exact-head CI, immutable diff and declared content receipts passed.", "owner_decisions": "none"}, "completed")
        return False  # Never reserve inference for this path.

    def accept_repository(self, job_id, receipt_path):
        """Explicit operator registration, never generated from a model report.

        The private file supplies head/source/criteria/reference/files as used by
        repository_verification. It neither asserts a deployment nor relaxes CI.
        """
        path = Path(receipt_path)
        with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK), "rb") as handle:
            info = os.fstat(handle.fileno())
            if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.geteuid()
                    or stat.S_IMODE(info.st_mode) & 0o077 or info.st_size > 65536):
                raise EvidenceError("private_acceptance_file_required")
            raw = handle.read(65537)
        if len(raw) > 65536:
            raise EvidenceError("private_acceptance_file_required")
        receipt = json.loads(raw)
        with self.store.connect() as db:
            db.execute("BEGIN IMMEDIATE")
            row = db.execute("SELECT * FROM jobs WHERE id=?", (job_id,)).fetchone()
            if row is None:
                raise EvidenceError("acceptance_job_unknown")
            job = dict(row)
            files = receipt.get("files") if isinstance(receipt, dict) else None
            if (not isinstance(receipt, dict) or set(receipt) != {"head", "source", "criteria", "reference", "files"}
                    or not SHA.fullmatch(str(receipt.get("head", ""))) or receipt["head"] != job["head"]
                    or receipt.get("source") not in ("owner_approved_artifact", "independent_review")
                    or receipt.get("criteria") != "exact_artifact_content"
                    or not isinstance(receipt.get("reference"), str) or not 1 <= len(receipt["reference"].strip()) <= 1000
                    or not isinstance(files, dict) or not files or len(files) > 100):
                raise EvidenceError("acceptance_declaration_invalid")
            for name, digest in files.items():
                if (not isinstance(name, str) or not name.startswith(("docs/", "journal/"))
                        or any(part in ("", ".", "..") for part in name.split("/"))
                        or any(name == p or name.startswith(p) for p in self.protected)
                        or (digest is not None and (not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest)))):
                    raise EvidenceError("acceptance_declaration_invalid")
            encoded = json.dumps(receipt, sort_keys=True, separators=(",", ":"))
            prior = db.execute("SELECT detail FROM events WHERE job=? AND kind='repository_acceptance' ORDER BY id DESC LIMIT 1", (job_id,)).fetchone()
            if prior and prior[0] == encoded:
                return False
            at = dt.datetime.now(dt.timezone.utc).isoformat()
            db.execute("INSERT INTO events(at,job,kind,detail) VALUES(?,?,'repository_acceptance',?)", (at, job_id, encoded))
            # Reconcile the same candidate only on newly supplied evidence.
            # Preserve its previous handoff/result; do not reserve inference.
            if job["state"] == "verification_pending" and job["merged"]:
                db.execute("UPDATE jobs SET state='deployment_wait',updated=? WHERE id=?", (at, job_id))
        self.next_poll.pop(job_id, None)
        return True

    def deployments(self, merged, merged_at):
        data = self.api('query($s:String!,$e:String!){serviceInstance(serviceId:$s,environmentId:$e){activeDeployments{id status createdAt meta} latestDeployment{id status createdAt meta}}}',
                        {"s": self.service, "e": self.environment})["serviceInstance"]
        active = data["activeDeployments"]
        if not isinstance(active, list) or len(active) > 20:
            raise EvidenceError("deployment_source_unavailable")
        rows = [row for row in [data.get("latestDeployment"), *active] if isinstance(row, dict)]
        # This is the authoritative active set, not a guess from historical
        # SUCCESS rows or the latest (possibly failed) build. No history needed
        # when it already identifies the exact revision or its replacement.
        successes = [row for row in active if row.get("status") == "SUCCESS"]
        if (any(self.deployment_sha(row) == merged for row in rows)
                or len(successes) == 1 and self.deployment_sha(successes[0])
                and clock(successes[0].get("createdAt")) > clock(merged_at)):
            return active, rows
        # If an unrelated active revision is returned, its timestamp is enough
        # to establish supersession only after comparison with this merge.
        # Include bounded target history so an older active revision cannot hide
        # a failed target behind a later unrelated build.
        after = None
        query = 'query($i:DeploymentListInput!,$after:String){deployments(first:100,after:$after,input:$i){edges{node{id status createdAt meta}} pageInfo{hasNextPage endCursor}}}'
        for _ in range(2):
            page = self.api(query, {"i": {"serviceId": self.service, "environmentId": self.environment}, "after": after})["deployments"]
            rows.extend(item["node"] for item in page["edges"])
            if any(self.deployment_sha(row) == merged for row in rows) or not page["pageInfo"]["hasNextPage"]:
                return active, rows
            after = page["pageInfo"]["endCursor"]
            if not isinstance(after, str) or not after:
                break
        raise EvidenceError("deployment_history_incomplete")

    @staticmethod
    def deployment_sha(row):
        meta = row.get("meta") or {}
        if isinstance(meta, str):
            meta = json.loads(meta)
        value = meta.get("commitHash")
        return value if isinstance(value, str) and SHA.fullmatch(value) else None

    def observe(self, job, verifying=False):
        try:
            plan = self.plan(job)
            if plan["scope"] == "repository_only":
                return self.repository_verification(job, plan)
            current, rows = self.deployments(job["merged"], plan["merged_at"])
            active = [row for row in current if row.get("status") == "SUCCESS"]
            exact = [row for row in rows if self.deployment_sha(row) == job["merged"]]
            # A newer FAILED/building deployment does not replace the active
            # SUCCESS. Ambiguous overlapping successes cannot certify identity.
            if len(active) == 1 and self.deployment_sha(active[0]) == job["merged"]:
                with urllib.request.urlopen("https://jiaoyi.up.railway.app/health", timeout=20) as response:
                    health = json.load(response)
                if not isinstance(health, dict) or not health:
                    raise EvidenceError("deployment_health_unavailable")
                self.record(job, "exact_deployment", {"deployment": active[0]["id"], "merged": job["merged"]})
                self.verified.add(job["id"])
                if not verifying and job["state"] != "verification_queued":
                    self.store.update(job["id"], "verification_queued")
                return True
            if (len(active) == 1 and self.deployment_sha(active[0])
                    and clock(active[0].get("createdAt")) > clock(plan["merged_at"])):
                self.handoff(job, "release_superseded", deployment=active[0]["id"], current_sha=self.deployment_sha(active[0]))
                return False
            if exact:
                latest = max(exact, key=lambda row: clock(row.get("createdAt")))
                if latest["status"] in ("FAILED", "CRASHED", "SKIPPED"):
                    self.handoff(job, "unexpected_runtime_skip" if latest["status"] == "SKIPPED" else "deployment_failed",
                                 deployment=latest["id"], deployment_status=latest["status"])
                    return False
            if verifying or dt.datetime.now(dt.timezone.utc) >= clock(plan["wait_deadline_at"]):
                self.handoff(job, "deployment_identity_unverified" if verifying else "deployment_evidence_timeout")
            return False
        except Exception as error:
            reason = str(error) if isinstance(error, EvidenceError) else "deployment_source_unavailable"
            plan = self.saved(job["id"], "release_scope") or {}
            # Missing identity/classification is a handoff now. Transport during
            # a known deployment wait is retried only inside its fixed deadline.
            retryable = reason in ("deployment_source_unavailable", "deployment_health_unavailable", "deployment_history_incomplete")
            if (verifying or not plan or not retryable
                    or dt.datetime.now(dt.timezone.utc) >= clock(plan["wait_deadline_at"])):
                self.handoff(job, reason)
            return False

    def poll(self):
        jobs = self.store.jobs(("ci_wait", "deployment_wait", "verification_queued"))
        due = [job for job in jobs if time.monotonic() >= self.next_poll.get(job["id"], 0)]
        if not due:
            return
        job = next((row for row in due if row["id"] > self.cursor), due[0])
        self.cursor = job["id"]
        self.next_poll[job["id"]] = time.monotonic() + POLL_SECONDS
        if job["state"] == "verification_queued":
            self.observe(job, verifying=True)
        else:
            self.c.release(job)

    def backlog(self):
        jobs = self.store.jobs(("ci_wait", "deployment_wait", "verification_queued", "verification_pending", "owner_blocked"))
        jobs = [job for job in jobs if job.get("head") or job.get("pr") or job.get("merged")]
        counts, items = {}, []
        for job in jobs:
            counts[job["state"]] = counts.get(job["state"], 0) + 1
            if len(items) >= 20:
                continue
            plan = self.saved(job["id"], "release_scope") or {}
            disposition = self.saved(job["id"], "release_disposition") or {}
            reason = disposition.get("release_reason")
            if reason is None and job["state"] == "owner_blocked":
                # Older coordinator guards stored this exact literal directly.
                # Never parse model prose or expose an unrecognised result.
                reason = ("release_identity_changed" if job.get("result") == "PR identity changed; release refused"
                          else "legacy_release_reason_unavailable")
            if job["state"] == "ci_wait" and not self.c.config.mutation_allowed():
                reason = "mutation_window_closed"
            items.append({"id": job["id"], "state": job["state"], "pr": job.get("pr"),
                "head": job.get("head"), "merged": job.get("merged"), "scope": plan.get("scope"),
                "reason": reason, "wait_deadline_at": plan.get("wait_deadline_at"),
                "owner_action_required": disposition.get("owner_action_required"), "updated": job["updated"]})
        try:
            oldest_age = max(0, (dt.datetime.now(dt.timezone.utc) - min(clock(job["created"]) for job in jobs)).total_seconds()) if jobs else None
        except EvidenceError:
            oldest_age = None
        return {"count": len(jobs), "counts": counts, "items": items, "oldest_age_seconds": oldest_age,
                "truncated": len(jobs) > len(items), "remaining_count": len(jobs) - len(items)}
