"""Bounded read-only sources for №18,393; no broker, trading or messaging calls.

The caller owns projection/freshness decisions. An HTTP 200 proves retrieval,
not that the account evidence inside the response is current. Workers are daemon
threads because DNS and third-party socket implementations can ignore timeout.
At most one worker per fixed source can remain alive across collection cycles.
"""
import datetime as dt
import http.client
import json
import math
import queue
import re
import threading
import time
import urllib.error
import urllib.parse
import urllib.request

MAX_BYTES = 5 * 1024 * 1024
DEFAULT_TIMEOUT = 4.2
MAX_TIMEOUT = 12.0
DEFAULT_BASE = "https://jiaoyi.up.railway.app"
RAILWAY_URL = "https://backboard.railway.com/graphql/v2"
RAILWAY_PROJECT = "26f609a2-afc0-4e94-90a2-fe87ba40a713"
RAILWAY_SERVICE = "0f67b4a0-629d-42f1-97f1-8910887896af"
RAILWAY_ENVIRONMENT = "f3b64526-67e3-4ea6-98af-274043a8b657"
RAILWAY_QUERY = """query($s:String!,$e:String!){
  serviceInstance(serviceId:$s,environmentId:$e){
    latestDeployment{id status createdAt meta}
    activeDeployments{id status createdAt meta}
  }
}"""
ROUTES = {
    "health": "/health",
    "setup_assessments": "/owner/setup-assessments",
    "operations_reports": "/operations/reports",
}
_ACTIVE = {}
_ACTIVE_LOCK = threading.Lock()


class NoRedirect(urllib.request.HTTPRedirectHandler):
    """Never forward owner/Railway credentials to any redirect target."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def _iso(value):
    if isinstance(value, dt.datetime):
        if value.tzinfo is None:
            raise ValueError("aware monitor timestamp required")
        return value.astimezone(dt.timezone.utc).isoformat()
    parsed = dt.datetime.fromisoformat(str(value).replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        raise ValueError("aware monitor timestamp required")
    return parsed.astimezone(dt.timezone.utc).isoformat()


def _base(env):
    value = env.get("TRADING_BASE_URL") or env.get("JIAOYI_BASE_URL") or DEFAULT_BASE
    try:
        parsed = urllib.parse.urlsplit(value)
        # Paths, credentials and queries could turn fixed routes into a secret
        # sink or leak into receipt URIs; only an HTTPS origin is accepted.
        if (parsed.scheme != "https" or not parsed.hostname or parsed.username
                or parsed.password or parsed.query or parsed.fragment
                or parsed.path not in ("", "/") or parsed.port == 0):
            return None
        return urllib.parse.urlunsplit(("https", parsed.netloc, "", "", ""))
    except (ValueError, TypeError):
        return None


def _receipt(source_uri, observed_at, **fields):
    return {"status": "unavailable", "source_uri": source_uri,
            "observed_at": observed_at, "elapsed_ms": 0,
            "freshness": "not_verified", **fields}


def _http_json(request, deadline):
    """Fixed errors only: never return URLs, headers or remote error bodies."""
    info = {"http_status": None, "bytes": None}
    try:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return None, {**info, "status": "timeout", "error_code": "deadline"}
        # Default HTTPSHandler preserves platform CA verification and hostname
        # checks. Do not install unverified TLS contexts or credential redirects.
        opener = urllib.request.build_opener(NoRedirect())
        with opener.open(request, timeout=remaining) as response:
            status = getattr(response, "status", None)
            info["http_status"] = status if type(status) is int and 100 <= status <= 599 else None
            if status != 200:
                return None, {**info, "status": "unavailable", "error_code": "http"}
            body = response.read(MAX_BYTES + 1)
        info["bytes"] = len(body)
        if len(body) > MAX_BYTES:
            return None, {**info, "status": "unavailable", "error_code": "oversize"}
        def invalid_constant(_):
            raise ValueError("non-finite JSON number")
        value = json.loads(body, parse_constant=invalid_constant)
        if not isinstance(value, dict):
            return None, {**info, "status": "unavailable", "error_code": "invalid_shape"}
        if time.monotonic() >= deadline:
            return None, {**info, "status": "timeout", "error_code": "deadline"}
        return value, {**info, "status": "ok", "error_code": None}
    except urllib.error.HTTPError as error:
        code = error.code if type(error.code) is int and 100 <= error.code <= 599 else None
        error.close()
        return None, {**info, "status": "unavailable", "http_status": code,
                      "error_code": "redirect_refused" if code and 300 <= code < 400 else "http"}
    except TimeoutError:
        return None, {**info, "status": "timeout", "error_code": "timeout"}
    except urllib.error.URLError as error:
        timed_out = isinstance(error.reason, TimeoutError)
        return None, {**info, "status": "timeout" if timed_out else "unavailable",
                      "error_code": "timeout" if timed_out else "transport"}
    except (json.JSONDecodeError, UnicodeDecodeError, RecursionError, ValueError):
        return None, {**info, "status": "unavailable", "error_code": "invalid_json"}
    except (OSError, http.client.HTTPException, TypeError):
        return None, {**info, "status": "unavailable", "error_code": "transport"}


def _deployment(request, deadline):
    value, status = _http_json(request, deadline)
    if value is None:
        return None, status
    data = value.get("data")
    instance = data.get("serviceInstance") if isinstance(data, dict) else None
    if value.get("errors") or not isinstance(instance, dict):
        return None, {**status, "status": "unavailable", "error_code": "graphql_unavailable"}
    def deployment(row):
        if not isinstance(row, dict):
            return None
        # The deployment meta bag can include build configuration. Only keep
        # the source revision needed for this monitor, never the raw bag.
        meta = row.get("meta")
        sha = meta.get("commitHash") if isinstance(meta, dict) else None
        return {"id": row.get("id"), "status": row.get("status"),
                "createdAt": row.get("createdAt"),
                "meta": {"commitHash": sha if isinstance(sha, str) and re.fullmatch(r"[a-fA-F0-9]{40}", sha) else None}}
    active = instance.get("activeDeployments")
    if not isinstance(active, list) or len(active) > 20 or any(not isinstance(row, dict) for row in active):
        return None, {**status, "status": "unavailable", "error_code": "deployment_shape"}
    return {"project_id": RAILWAY_PROJECT, "service_id": RAILWAY_SERVICE,
            "environment_id": RAILWAY_ENVIRONMENT,
            "latestDeployment": deployment(instance.get("latestDeployment")),
            "activeDeployments": [deployment(row) for row in active]}, status


def _sql(env, deadline, since, observed_at):
    from .sql_evidence import collect_sql_evidence
    value = collect_sql_evidence(env, deadline=deadline, since=since, now=observed_at)
    if not isinstance(value, dict):
        return None, {"status": "unavailable", "error_code": "invalid_shape"}
    status = value.get("status", "ok")
    if status not in ("ok", "available", "complete", "partial"):
        # SQL returns fixed reason codes; unknown text is never propagated.
        reason = value.get("reason")
        safe_reasons = {"evidence_output_limit", "readonly_query_failed", "readonly_query_timeout",
                        "readonly_evidence_invalid", "readonly_psql_unavailable",
                        "readonly_query_deadline_elapsed", "invalid_monitor_window"}
        return value, {"status": "unavailable", "error_code": reason if reason in safe_reasons else "sql_unavailable"}
    return value, {"status": "ok", "error_code": None}


def collect_sources(env, started_at, observed_at, timeout=DEFAULT_TIMEOUT):
    """Collect within 4.2s by default, or an explicit budget capped at 12s.

    started_at and observed_at are aware UTC datetimes or ISO timestamps. The
    former selects the SQL evidence window; neither controls the deadline clock.
    Scheduled checkpoints may request 12s for the existing slow health source;
    fresh --once reads retain the default 4.2s. SQL remains capped at 3.5s.
    Dead workers are reaped; still-blocked workers occupy their fixed slot and
    later calls report busy instead of accumulating additional stuck threads.
    """
    if not isinstance(timeout, (int, float)) or not math.isfinite(timeout) or timeout <= 0:
        raise ValueError("positive finite collection timeout required")
    begun = time.monotonic()
    deadline = begun + min(timeout, MAX_TIMEOUT)
    since, observed = _iso(started_at), _iso(observed_at)
    source_env = dict(env)
    base = _base(source_env)
    result = {name: None for name in (*ROUTES, "sql_evidence", "deployment")}
    receipts, jobs = {}, {}
    for name, route in ROUTES.items():
        uri = (base + route) if base else "unconfigured_https_origin" + route
        receipts[name] = _receipt(uri, observed)
        if base is None:
            receipts[name]["error_code"] = "invalid_base_url"
            continue
        headers = {"Accept": "application/json", "Cache-Control": "no-cache",
                   "User-Agent": "jiaoyi-live-monitor/18393"}
        if name == "setup_assessments":
            password = source_env.get("JIAOYI_UI_PASSWORD")
            if not password:
                receipts[name]["error_code"] = "owner_credential_missing"
                continue
            headers["Authorization"] = "Bearer " + password
        request = urllib.request.Request(uri, headers=headers, method="GET")
        jobs[name] = lambda request=request: _http_json(request, deadline)

    receipts["deployment"] = _receipt(RAILWAY_URL, observed, project_id=RAILWAY_PROJECT,
        service_id=RAILWAY_SERVICE, environment_id=RAILWAY_ENVIRONMENT)
    token = source_env.get("RAILWAY_TOKEN")
    if token:
        request = urllib.request.Request(RAILWAY_URL, method="POST", headers={
            "Content-Type": "application/json", "Project-Access-Token": token,
            "User-Agent": "jiaoyi-live-monitor/18393"}, data=json.dumps({
                "query": RAILWAY_QUERY,
                "variables": {"s": RAILWAY_SERVICE, "e": RAILWAY_ENVIRONMENT},
            }).encode())
        jobs["deployment"] = lambda request=request: _deployment(request, deadline)
    else:
        receipts["deployment"]["error_code"] = "railway_credential_missing"

    receipts["sql_evidence"] = _receipt("postgresql:LIVE_MONITOR_DATABASE_URL", observed)
    if source_env.get("LIVE_MONITOR_DATABASE_URL"):
        jobs["sql_evidence"] = lambda: _sql(source_env, min(deadline, begun + 3.5), since, observed)
    else:
        receipts["sql_evidence"]["error_code"] = "database_credential_missing"

    completed = queue.Queue()
    pending = set()

    def work(name, job):
        source_start = time.monotonic()
        try:
            try:
                value, status = job()
            except Exception:
                # Third-party errors may embed passwords, request objects or
                # remote payloads. Keep only a fixed, inspectable failure code.
                value, status = None, {"status": "unavailable", "error_code": "source_failed"}
            finished = time.monotonic()
            completed.put((name, value, {**status,
                "elapsed_ms": max(0, round((finished-source_start)*1000))}, finished))
        finally:
            with _ACTIVE_LOCK:
                if _ACTIVE.get(name) is threading.current_thread():
                    del _ACTIVE[name]

    for name, job in jobs.items():
        with _ACTIVE_LOCK:
            active = _ACTIVE.get(name)
            if active is not None and active.is_alive():
                receipts[name].update(status="busy", error_code="previous_source_still_running")
                continue
            thread = threading.Thread(target=work, args=(name, job),
                                      name="jiaoyi-monitor-" + name, daemon=True)
            _ACTIVE[name] = thread
            pending.add(name)
            try:
                thread.start()
            except RuntimeError:
                del _ACTIVE[name]
                pending.remove(name)
                receipts[name].update(status="unavailable", error_code="worker_start_failed")
    def accept(item):
        name, value, status, finished = item
        if name not in pending:
            return
        pending.remove(name)
        if finished >= deadline:
            receipts[name].update(status="timeout", error_code="deadline",
                                  elapsed_ms=round((deadline-begun)*1000))
        else:
            result[name] = value
            receipts[name].update(status)
    while pending:
        remaining = deadline-time.monotonic()
        if remaining <= 0:
            break
        try:
            item = completed.get(timeout=remaining)
        except queue.Empty:
            break
        accept(item)
    # Preserve results finished before the deadline but not yet dequeued; do
    # not wait for workers or accept a late completion as a current snapshot.
    for _ in range(len(pending)):
        try:
            accept(completed.get_nowait())
        except queue.Empty:
            break
    for name in pending:
        receipts[name].update(status="timeout", error_code="deadline",
                              elapsed_ms=max(0, round((time.monotonic()-begun)*1000)))
    result["source_status"] = receipts
    return result
