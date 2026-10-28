"""Opt-in, one-off market validation using private bounded evidence and no model."""
import datetime as dt
import hashlib
import html
import json
import math
import re
import threading
from pathlib import Path
import urllib.request

SGT = dt.timezone(dt.timedelta(hours=8))
ROUTES = ("/health", "/operations/reports", "/operations/trailing-receipts", "/operations/stop-loss-reviews")
MAX_SEEN = 256
TRAIL_CHECKS = ("owned_parent", "precision_known", "original_r_matches", "request_matches_plan",
    "request_uses_original_r", "stop_improves", "broker_acknowledged", "broker_stop_matches", "take_profit_preserved")


def _clock(value=None):
    if value is None:
        value = dt.datetime.now(dt.timezone.utc)
    if isinstance(value, str):
        value = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    if not isinstance(value, dt.datetime) or value.tzinfo is None:
        raise ValueError("validation clock must be timezone-aware")
    return value.astimezone(dt.timezone.utc)


def _number(value):
    return value if type(value) in (int, float) and math.isfinite(value) else None


def _source_clock(value):
    if value is None:
        raise ValueError("source clock is missing")
    return _clock(value)


def _text(value, limit=180):
    return str(value)[:limit] if value is not None else "unknown"


def _stamp(value):
    try:
        return _clock(value).astimezone(SGT).strftime("%d %b %Y %H:%M:%S SGT") if value else "unknown"
    except (ValueError, TypeError):
        return "unknown"


def _goals(operations):
    out = {}
    for row in operations.get("reports", [])[:16]:
        if not isinstance(row, dict):
            continue
        assessment = ((row.get("report") or {}).get("assessment") or {})
        window = assessment.get("window") or {}
        count, wins = window.get("count"), window.get("wins")
        qualified = (assessment.get("history_complete") is True and type(count) is int
                     and count == 20 and type(wins) is int and 0 <= wins <= 20
                     and row.get("monitor_state") == "alive")
        key = _text(row.get("account_id")) + "/" + _text(row.get("strategy"))
        out[key] = {"state": assessment.get("state", "unknown"), "qualified": qualified,
                    "count": count if type(count) is int else None,
                    "wins": wins if type(wins) is int else None,
                    "net_r": _number(window.get("net_r")),
                    "payoff": _number(window.get("realized_payoff")),
                    "checked_at": row.get("checked_at")}
    return out


def _project(sources, worker):
    health = sources.get("/health") or {}
    operations = sources.get("/operations/reports") or {}
    loops, preparation, quotes = [], [], []
    for section, strategy in (("trading", "s1"), ("strategy3", "s3")):
        for row in health.get(section, [])[:8]:
            if not isinstance(row, dict):
                continue
            live = row.get("liveness") or {}
            fast = live.get("fast_monitor") or {}
            loops.append({"account": row.get("account_id"), "strategy": strategy,
                "state": live.get("state"), "monitor": fast.get("state"),
                "started_at": live.get("started_at"), "last_quote_at": live.get("last_quote_at"),
                "exit_checks": fast.get("exit_checks_completed"),
                "watchdog_age_ms": _number(fast.get("watchdog_completion_age_ms")),
                "unresolved_open": fast.get("unresolved_open_count"),
                "connected": fast.get("session_connected")})
            if strategy != "s3":
                continue
            for item in ((row.get("preopen_preparation") or {}).get("symbols") or [])[:64]:
                if isinstance(item, dict) and item.get("symbol") in ("AUS200", "JPN225", "BTCUSD", "ETHUSD"):
                    preparation.append({"account": row.get("account_id"), "prereg_hash": row.get("prereg_hash"),
                        **{key: item.get(key) for key in ("symbol", "session_open", "session_close", "deadline_at",
                            "state", "completed_at", "deadline_met", "source_cutoff_at", "history_ready",
                            "spread_ready", "commission_ready", "metadata_ready", "missing_marks")}})
            for symbol, quote in list((row.get("trigger_distance") or {}).items())[:32]:
                if symbol in ("AUS200", "JPN225", "BTCUSD", "ETHUSD") and isinstance(quote, dict):
                    quotes.append({"account": row.get("account_id"), "symbol": symbol,
                        "at": quote.get("at"), "bid": _number(quote.get("bid")),
                        "ask": _number(quote.get("ask")), "reason": _text(quote.get("reason"), 160)})
    receipts = []
    for row in (sources.get("/operations/trailing-receipts") or {}).get("receipts", [])[:32]:
        if isinstance(row, dict):
            receipts.append({key: row.get(key) for key in ("account_id", "entry_key", "position_id", "symbol",
                "state", "decided_at", "acknowledged_at", "checks")})
    cases = []
    for row in (sources.get("/operations/stop-loss-reviews") or {}).get("cases", [])[:128]:
        if isinstance(row, dict):
            cases.append({key: row.get(key) for key in ("case_id", "closed_at", "ownership_verified", "missing_evidence")})
    return {"health": health.get("status", "unknown"), "loops": loops, "preparation": preparation,
        "quotes": quotes, "goals": _goals(operations), "receipts": receipts, "sl_cases": cases,
        "worker": {key: worker.get(key) for key in ("state", "sessions_today_sgt", "daily_session_limit",
            "worker_bundle_sha256", "release_commit", "engineering_work_until", "engineering_mutations_allowed")},
        "incidents": {"counts": (worker.get("incidents") or {}).get("counts", {}),
            "last_observed_at": (worker.get("incidents") or {}).get("last_observed_at"),
            "batch_sizes": [item.get("analysis_batch_size") for item in (worker.get("incidents") or {}).get("incidents", [])
                if type(item.get("analysis_batch_size")) is int]},
        "source_clocks": {route: value.get("as_of") for route, value in sources.items() if isinstance(value, dict)}}


def render(state, now=None):
    now = _clock(now)
    esc = lambda value: html.escape(_text(value, 2200), quote=True)
    latest, baseline = state.get("latest") or {}, state.get("baseline") or {}
    monitor = state.get("monitor", {})
    verdict = ("FAIL" if monitor.get("unhealthy") else "UNKNOWN" if not monitor.get("samples")
               or monitor.get("unknown_samples") or state.get("failures", {}).get("/health") else "PASS")
    trail = state.get("trailing", {})
    trail_state = "FAIL" if trail.get("refused") else "PASS" if trail.get("passed") else "NOT EXERCISED"
    if trail.get("unknown") and not trail.get("refused"):
        trail_state = "UNKNOWN" if not trail.get("passed") else "PASS / partial coverage"
    goal_rows = []
    for key, item in (latest.get("goals") or {}).items():
        old = (baseline.get("goals") or {}).get(key) or {}
        comparable = (item.get("qualified") and old.get("qualified") and
                      (latest.get("worker") or {}).get("release_commit") == (baseline.get("worker") or {}).get("release_commit")
                      and (latest.get("worker") or {}).get("release_commit") is not None)
        delta = ("%+d net wins" % (item["wins"] - old["wins"])) if comparable else "unknown"
        gap = str(max(0, 15 - item["wins"])) + " wins" if item.get("qualified") else "unknown"
        wins = str(item.get("wins", "?")) + "/" + str(item.get("count", "?"))
        goal_rows.append("<tr>" + "".join("<td>" + esc(value) + "</td>" for value in
            (key, wins, gap, item.get("payoff"), delta)) + "</tr>")
    prep_rows = []
    for item in state.get("preparation", {}).values():
        met = (item.get("deadline_met") is True and item.get("completed_at") and item.get("deadline_at"))
        if met:
            try:
                met = _clock(item["completed_at"]) <= _clock(item["deadline_at"])
            except (TypeError, ValueError):
                met = False
        result = "PASS" if met else "UNKNOWN"
        try:
            watched = _source_clock(item["first_observed_at"]) <= _source_clock(item["deadline_at"])
            late_receipt = item.get("completed_at") and _source_clock(item["completed_at"]) > _source_clock(item["deadline_at"])
            if not met and late_receipt:
                result = "FAIL"
            elif not met and watched and now >= _source_clock(item["deadline_at"]):
                result = "FAIL" if item.get("state") in ("prepared_late", "incomplete", "deadline_missed") else "UNKNOWN"
        except (KeyError, ValueError, TypeError):
            pass
        if item.get("registration_changed"):
            result = "UNKNOWN"
        prep_rows.append("<tr>" + "".join("<td>" + esc(value) + "</td>" for value in
            (str(item.get("account")) + "/" + str(item.get("symbol")), result,
             _stamp(item.get("session_open")), _stamp(item.get("deadline_at")),
             _stamp(item.get("completed_at")), item.get("state"))) + "</tr>")
    worker = latest.get("worker") or {}
    quote_rows, chosen = [], {}
    for quote in latest.get("quotes", []):
        old = chosen.get(quote.get("symbol"))
        if old is None or str(quote.get("at") or "") > str(old.get("at") or ""):
            chosen[quote.get("symbol")] = quote
    for symbol, quote in sorted(chosen.items()):
        freshness = "UNKNOWN"
        try:
            freshness = "fresh" if 0 <= (now - _source_clock(quote.get("at"))).total_seconds() <= 120 else "stale"
        except (ValueError, TypeError):
            pass
        quote_rows.append("<tr>" + "".join("<td>" + esc(value) + "</td>" for value in
            (symbol, quote.get("account"), quote.get("bid"), quote.get("ask"), _stamp(quote.get("at")), freshness)) + "</tr>")
    lifecycle = "<p>Private statement/lifecycle evidence is unavailable.</p>"
    try:
        import market_validation_evidence
        lifecycle = market_validation_evidence.render(state.get("private_evidence") or {})
    except Exception:
        pass
    serial = "V-" + state["key"][:12]
    number = 0
    def para(text):
        nonlocal number
        number += 1
        return "<p>§" + str(number) + ". " + text + "</p>"
    document = ("<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'>"
        "<title>Jiaoyi market validation</title><style>body{font:15px/1.5 system-ui;max-width:1000px;margin:24px auto;padding:0 16px;color:#172838}"
        "h1{font-size:24px}h2{font-size:18px}table{border-collapse:collapse;width:100%;font-size:13px}td,th{padding:7px;text-align:left;border-bottom:1px solid #ddd}.goal{background:#edf4fc;padding:12px}p{overflow-wrap:anywhere}</style>"
        "<h1>09:00 SGT market/fix validation</h1>"
        + para("Generated " + esc(_stamp(now.isoformat())) + "; requested " + esc(_stamp(state["due_at"]))
            + ". Observed window " + esc(_stamp(state.get("started_at"))) + " → " + esc(_stamp(state.get("last_sample_at")))
            + ". Report ID " + serial + ". by &lt;Codex (GPT-6), deterministic reporter; no model call&gt;. Chat: " + esc(state.get("chat_references")))
        + "<div class=goal>" + para("<b>Goal:</b> each strategy/account's latest20 fully closed owned positions; ≥73% net wins (15/20) and risk:reward1:1.38. "
            "The table reports measured window changes only. Overlapping windows are not independent; financial improvement caused by a software fix remains unproved.") + "</div>"
        + "<table><tr><th>Account/strategy</th><th>Net wins</th><th>Gap to15/20</th><th>Realized payoff</th><th>Baseline change</th></tr>" + "".join(goal_rows) + "</table>"
        + "<h2>Market observations</h2><table><tr><th>Instrument</th><th>Quote account</th><th>Bid</th><th>Ask</th><th>Source clock</th><th>Freshness</th></tr>"
        + ("".join(quote_rows) or "<tr><td colspan=6>UNKNOWN: no curated broker quote available.</td></tr>") + "</table>"
        + "<h2>Observed fixes</h2>"
        + para("<b>Monitoring: " + verdict + "</b>; " + esc(monitor.get("samples", 0)) + " sampled observations, "
            + esc(monitor.get("unhealthy", 0)) + " unhealthy samples. Maximum observed watchdog completion age: " + esc(monitor.get("max_watchdog_age_ms")) + "ms. "
            "Sampling is not a continuous latency/protection certificate; absent snapshots are source gaps.")
        + para("<b>New trailing evidence: " + trail_state + "</b>; observed passed=" + esc(trail.get("passed", 0))
            + ", refused=" + esc(trail.get("refused", 0)) + ", unknown=" + esc(trail.get("unknown", 0))
            + ". Only new identities after the baseline are counted. Passed verifies correlated broker acknowledgement, originalR arithmetic and TP preservation. "
            "No observed new amendment means NOT EXERCISED; historical receipts and restarts are not new live proof.")
        + para("<b>Observed new owned SL review cases:</b> " + esc(state.get("sl_new_cases", 0))
            + ". This bounded aggregate is not a whole-account close count; broker close reasons and original setup/SL/TP evidence are assessed below.")
        + para("<b>Worker:</b> " + esc(worker.get("state")) + "; sessions " + esc(worker.get("sessions_today_sgt")) + "/" + esc(worker.get("daily_session_limit"))
            + "; bundle " + esc(worker.get("worker_bundle_sha256")) + "; engineering mutations allowed at observation=" + esc(worker.get("engineering_mutations_allowed"))
            + ". Observed batch sizes: " + esc((latest.get("incidents") or {}).get("batch_sizes", [])) + ". Read-only reporting does not consume an AI session.")
        + "<h2>Preparation before market open</h2><table><tr><th>Account/symbol</th><th>Verdict</th><th>Open</th><th>Deadline</th><th>Completion</th><th>Source state</th></tr>"
        + ("".join(prep_rows) or "<tr><td colspan=6>UNKNOWN: no applicable preparation receipts observed.</td></tr>") + "</table>"
        + para("Preparation uses the source receipt's completion time and ten-minute deadline. A late-started observation cannot prove an earlier miss. Actual opening price/costs/margin and ownership remain entry-time checks.")
        + lifecycle + "<h2>Coverage and next action</h2>"
        + para("Latest source clocks: " + "; ".join(esc(route) + "=" + esc(_stamp(value)) for route, value in latest.get("source_clocks", {}).items())
            + ". Source clocks do not become the report creation clock.")
        + para("Source failures: " + esc(state.get("failures", {})) + ". Reported endpoint views are bounded, so event counts are observed minimums; deduplication capacity reached=" + esc(state.get("coverage_limit_reached", False)) + ". "
            "Missing statement, ownership, SL/TP, closure or reverse proof remains UNKNOWN. Individual stop losses stay internal. "
            "This report does not place orders, reverse trades, loosen stops, retune rules or restart engineering after its07:30SGT cutoff.") + "</html>")
    number = 0
    def numbered(match):
        nonlocal number
        number += 1
        return "<p>§" + str(number) + ". "
    return re.sub(r"<p>(?:§\d+\. )?", numbered, document)


class MarketValidation:
    def __init__(self, coordinator):
        self.c = coordinator
        self.store = coordinator.store
        self.env = coordinator.config.env
        self.lock = threading.RLock()
        self.enabled = False
        self._state = {}
        self.config_error = None
        value = self.env.get("ENGINEERING_VALIDATION_REPORT_AT")
        if not value:
            return
        try:
            self.due = _clock(value)
        except (ValueError, TypeError):
            self.config_error = "invalid_aware_report_time"
            return
        key = hashlib.sha256(self.due.isoformat().encode()).hexdigest()
        self.path = self.store.private / ("market-validation-" + key[:20] + ".json")
        if self.path.is_symlink():
            raise RuntimeError("validation state cannot be a symlink")
        if self.path.exists():
            self._state = json.loads(self.path.read_text())
            if self._state.get("key") != key:
                raise ValueError("validation schedule identity differs")
        else:
            self._state = {"key": key, "due_at": self.due.isoformat(), "state": "waiting", "started_at": None,
                "last_sample_at": None, "observations": 0, "failures": {}, "baseline": None, "latest": None,
                "monitor": {"samples": 0, "unhealthy": 0, "unknown_samples": 0, "max_watchdog_age_ms": None},
                "trailing": {"passed": 0, "refused": 0, "unknown": 0}, "seen_trails": [], "seen_sl": [],
                "trail_states": {},
                "sl_new_cases": 0, "preparation": {}, "message_id": None,
                "chat_references": self.env.get("ENGINEERING_VALIDATION_CHAT_REFERENCES", "not supplied")[:1600]}
            self._persist()
            self.store.event("market_validation_scheduled", detail=json.dumps({"key": key, "due_at": self.due.isoformat()}))
        self.enabled = True

    def _persist(self):
        raw = json.dumps(self._state, sort_keys=True, allow_nan=False)
        if len(raw.encode()) > 512 * 1024:
            raise ValueError("validation evidence exceeds private bound")
        temporary = self.path.with_suffix(".tmp")
        if temporary.is_symlink():
            raise RuntimeError("validation temporary path cannot be a symlink")
        temporary.write_text(raw)
        temporary.chmod(0o600)
        temporary.replace(self.path)

    def _fetch(self, route):
        if route not in ROUTES:
            raise ValueError("unapproved validation source")
        request = urllib.request.Request("https://jiaoyi.up.railway.app" + route, headers={"Cache-Control": "no-cache"})
        with urllib.request.urlopen(request, timeout=8) as response:
            raw = response.read(2 * 1024 * 1024 + 1)
        if len(raw) > 2 * 1024 * 1024:
            raise ValueError("validation source too large")
        value = json.loads(raw)
        if not isinstance(value, dict):
            raise ValueError("validation source must be an object")
        return value

    def _collect(self, clock):
        sources, errors = {}, {}
        for route in ROUTES:
            try:
                sources[route] = self._fetch(route)
            except Exception as error:
                errors[route] = type(error).__name__
        return _project(sources, self.c.status()), errors

    def _evidence(self, clock):
        try:
            import market_validation_evidence
            return market_validation_evidence.collect(self.c, self._state.get("started_at") or clock.isoformat(), clock.isoformat())
        except Exception as error:
            return {"state": "unknown", "statement_report": {}, "accounts": [], "lifecycle": [],
                    "reason": "Private evidence collection unavailable: " + type(error).__name__}

    def _apply(self, observation, errors, clock):
        state = self._state
        first = state.get("baseline") is None
        if first:
            state["started_at"] = clock.isoformat()
            state["baseline"] = observation
        state["latest"] = observation
        state["last_sample_at"] = clock.isoformat()
        state["observations"] += 1
        for route in errors:
            state["failures"][route] = state["failures"].get(route, 0) + 1
        loops = observation["loops"]
        if observation["health"] not in ("ok", "unknown"):
            state["monitor"]["unhealthy"] += 1
        if loops and observation["health"] == "ok":
            state["monitor"]["samples"] += 1
            bad = any(row["state"] in ("stalled", "error", "dead") or row["monitor"] in
                ("stalled", "overdue", "error", "disconnected", "watchdog_stalled", "actor_work_overdue")
                or row.get("connected") is False or (row.get("unresolved_open") or 0) > 0 for row in loops)
            state["monitor"]["unhealthy"] += int(bad)
            complete = (len(loops) == 8 and len({(row["account"], row["strategy"]) for row in loops}) == 8
                        and all(row["state"] == "alive" and row["monitor"] == "observing"
                                and row["connected"] is True and row["unresolved_open"] == 0 for row in loops))
            state["monitor"]["unknown_samples"] = state["monitor"].get("unknown_samples", 0) + int(not complete and not bad)
            ages = [row["watchdog_age_ms"] for row in loops if row["watchdog_age_ms"] is not None]
            if ages:
                old = state["monitor"].get("max_watchdog_age_ms")
                state["monitor"]["max_watchdog_age_ms"] = max(ages + ([old] if old is not None else []))
        for row in observation["preparation"]:
            try:
                opened = _source_clock(row["session_open"])
            except (ValueError, TypeError):
                continue
            if not _clock(state["started_at"]) <= opened <= self.due:
                continue
            key = str(row["account"]) + ":" + row["symbol"] + ":" + opened.isoformat()
            old = state["preparation"].get(key) or {}
            state["preparation"][key] = {**row, "first_observed_at": old.get("first_observed_at", clock.isoformat()),
                "last_observed_at": clock.isoformat(),
                "registration_changed": bool(old.get("registration_changed") or
                    (old.get("prereg_hash") and row.get("prereg_hash") != old["prereg_hash"]))}
        for receipt in observation["receipts"]:
            if not isinstance(receipt.get("entry_key"), str):
                continue
            key = str(receipt.get("account_id")) + ":" + str(receipt.get("entry_key"))
            when = receipt.get("acknowledged_at") or receipt.get("decided_at")
            try:
                new = _clock(state["started_at"]) <= _source_clock(when) <= clock
            except (ValueError, TypeError):
                new = False
            if not first and new:
                result = receipt.get("state")
                result = result if result in ("passed", "refused", "unknown") else "unknown"
                if result == "passed" and not all((receipt.get("checks") or {}).get(check) is True for check in TRAIL_CHECKS):
                    result = "unknown"
                old = state.setdefault("trail_states", {}).get(key)
                if old != result:
                    if old is None and len(state["trail_states"]) >= MAX_SEEN:
                        state["coverage_limit_reached"] = True
                        continue
                    if old:
                        state["trailing"][old] -= 1
                    state["trailing"][result] += 1
                    state["trail_states"][key] = result
            if key not in state["seen_trails"]:
                state["seen_trails"] = (state["seen_trails"] + [key])[-MAX_SEEN:]
        for case in observation["sl_cases"]:
            key = case.get("case_id")
            if not isinstance(key, str) or key in state["seen_sl"]:
                continue
            state["seen_sl"] = (state["seen_sl"] + [key])[-MAX_SEEN:]
            try:
                new = _clock(state["started_at"]) <= _source_clock(case.get("closed_at")) <= clock
            except (ValueError, TypeError):
                new = False
            if not first and new and case.get("ownership_verified") is True:
                state["sl_new_cases"] += 1

    def tick(self, clock=None):
        if not self.enabled:
            return self.status()
        clock = _clock(clock)
        with self.lock:
            if self._state["state"] in ("sending", "delivered", "uncertain", "failed"):
                return self.status()
            last = self._state.get("last_sample_at")
            sample = not last or (clock - _clock(last)).total_seconds() >= 60 or clock >= self.due
        if sample:
            observation, errors = self._collect(clock)
            with self.lock:
                first = self._state.get("baseline") is None
                self._apply(observation, errors, clock)
                self._persist()
            if first or clock >= self.due or (self._state.get("private_evidence") or {}).get("state") != "available":
                evidence = self._evidence(clock)
                with self.lock:
                    self._state["private_evidence"] = evidence
                    self._persist()
        if clock >= self.due:
            self._deliver(clock)
        return self.status()

    def _deliver(self, clock):
        from reporting import DeliveryError, send_document
        with self.lock:
            if self._state["state"] in ("sending", "delivered", "uncertain", "failed"):
                return
            if not self.env.get("TELEGRAM_BOT_TOKEN") or not self.env.get("TELEGRAM_OWNER_CHAT_ID"):
                self._state["state"] = "failed"
                self._state["last_error"] = "telegram_not_configured"
                self._persist()
                return
            document = render(self._state, clock)
            path = self.path.with_suffix(".html")
            path.write_text(document)
            path.chmod(0o600)
            self._state.update(state="sending", report_path=str(path), report_sha256=hashlib.sha256(document.encode()).hexdigest())
            self._persist()
            self.store.event("market_validation_sending", detail=json.dumps({"key": self._state["key"], "due_at": self._state["due_at"]}))
        try:
            message = send_document(self.env["TELEGRAM_BOT_TOKEN"], self.env["TELEGRAM_OWNER_CHAT_ID"], path.name,
                document, "Jiaoyi · 09:00 SGT market/fix validation · HTML report")
            result = {"state": "delivered", "message_id": message, "delivered_at": _clock().isoformat()}
        except DeliveryError as error:
            result = {"state": "uncertain" if error.uncertain else "failed", "last_error": str(error)[:120]}
        except Exception as error:
            result = {"state": "uncertain", "last_error": type(error).__name__}
        with self.lock:
            self._state.update(result)
            self._persist()
            self.store.event("market_validation_delivery", detail=json.dumps({"key": self._state["key"], **result}))

    def status(self):
        with self.lock:
            if not self.enabled:
                return {"configured": False, "state": self.config_error or "disabled"}
            evidence = self._state.get("private_evidence") or {}
            statement = evidence.get("statement_report") or {}
            accounts = evidence.get("accounts") or []
            period = None
            try:
                period = dt.date.fromisoformat(str(statement.get("period_to"))[:10]).isoformat()
            except (TypeError, ValueError):
                pass
            return {"configured": True, "state": self._state["state"], "report_at": self._state["due_at"],
                "baseline_at": self._state.get("started_at"), "last_sample_at": self._state.get("last_sample_at"),
                "observations": self._state.get("observations", 0), "source_failure_count": sum(self._state.get("failures", {}).values()),
                "message_id": self._state.get("message_id"), "model_calls": 0, "sample_seconds": 60,
                "private_evidence_state": evidence.get("state", "not_collected") if evidence.get("state", "not_collected") in
                    ("available", "partial", "unknown", "not_collected") else "unknown",
                "private_source_error": evidence.get("reason") if re.fullmatch(r"private_readonly_[A-Za-z0-9_]{1,100}", str(evidence.get("reason", ""))) else None,
                "account_snapshot_count": sum(bool(row.get("observed_at")) for row in accounts if isinstance(row, dict)) if isinstance(accounts, list) else 0,
                "statement_report_available": statement.get("status") == "sent" and bool(statement.get("report_key")),
                "statement_report_period": period}

    def run(self):
        while self.enabled and not self.c.shutdown.is_set():
            try:
                self.tick()
            except Exception as error:
                with self.lock:
                    code = type(error).__name__
                    if self._state.get("last_error") != code:
                        self._state["last_error"] = code
                        self._persist()
                        self.store.event("market_validation_error", detail=code)
            if self.status().get("state") in ("delivered", "uncertain", "sending", "failed"):
                return
            self.c.shutdown.wait(30)
