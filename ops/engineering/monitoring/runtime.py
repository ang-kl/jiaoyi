"""Durable 12-checkpoint schedule and private delivery for directive 18,393."""
import base64
import datetime as dt
import fcntl
import hashlib
import hmac
import json
import math
import os
from pathlib import Path
import re
import threading
from urllib.parse import urlsplit

UTC = dt.timezone.utc
SGT = dt.timezone(dt.timedelta(hours=8))
MAX_LOG_BYTES = 5 * 1024 * 1024


def clock(value=None):
    value = dt.datetime.now(UTC) if value is None else value
    if isinstance(value, str):
        value = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    if not isinstance(value, dt.datetime) or value.tzinfo is None:
        raise ValueError("an aware timestamp is required")
    return value.astimezone(UTC)


def encoded(value):
    return json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":")).encode()


def atomic(path, data):
    temporary = path.with_name(path.name + ".tmp")
    with open(temporary, "wb") as output:
        os.chmod(temporary, 0o600)
        output.write(data)
        output.flush()
        os.fsync(output.fileno())
    os.replace(temporary, path)
    descriptor = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


class Monitor:
    def __init__(self, env, root):
        self.env = dict(env)
        self.enabled = self.env.get("LIVE_MONITOR_ENABLED") == "1"
        self.run_id = self.env.get("LIVE_MONITOR_RUN_ID", "18393-20261109")
        if not re.fullmatch(r"[A-Za-z0-9_.-]{1,80}", self.run_id):
            raise ValueError("invalid monitoring run identity")
        self.hours = float(self.env.get("LIVE_MONITOR_HOURS", "24"))
        self.interval = float(self.env.get("LIVE_MONITOR_INTERVAL_HOURS", "2"))
        if not all(math.isfinite(x) for x in (self.hours, self.interval)):
            raise ValueError("non-finite schedule")
        if not (0 < self.interval <= self.hours <= 168):
            raise ValueError("invalid bounded schedule")
        self.count = round(self.hours / self.interval)
        if not (1 <= self.count <= 48) or not math.isclose(self.count * self.interval, self.hours):
            raise ValueError("duration must be an integral number of intervals, at most 48")
        self.root = Path(root) / self.run_id
        self.plan_path = self.root / "plan.json"
        self.log_path = self.root / "docs" / "live_monitoring_24h_log.jsonl"
        self.completion_path = self.root / "completion.json"
        self.last_error = None

    def _plan(self):
        if not self.plan_path.exists():
            return None
        plan = json.loads(self.plan_path.read_text())
        if (plan.get("run_id"), plan.get("hours"), plan.get("interval_hours"), plan.get("checkpoints")) != (
            self.run_id, self.hours, self.interval, self.count
        ):
            raise ValueError("persisted schedule differs; use an explicitly new run identity")
        clock(plan["started_at"])
        return plan

    def _records(self):
        if not self.log_path.exists():
            return []
        if self.log_path.stat().st_size > MAX_LOG_BYTES:
            raise ValueError("checkpoint file exceeds bounded size")
        rows = [json.loads(line) for line in self.log_path.read_text().splitlines() if line]
        if len(rows) > self.count:
            raise ValueError("too many checkpoints")
        for index, row in enumerate(rows, 1):
            if row.get("checkpoint_number") != index or row.get("run_id") != self.run_id:
                raise ValueError("checkpoint sequence or identity mismatch")
        return rows

    def _capture(self, plan, at, checkpoint_number, previous, *, timeout=12.0):
        from .projection import build_snapshot
        from .sources import collect_sources
        start = clock(plan["started_at"])
        end = start + dt.timedelta(hours=self.hours)
        cutoff = min(at, end)
        source = collect_sources(self.env, start, cutoff, timeout=timeout)
        completed = clock()
        result = build_snapshot(
            health=source.get("health"), setup_assessments=source.get("setup_assessments"),
            operations_reports=source.get("operations_reports"), sql_evidence=source.get("sql_evidence"),
            started_at=start, observed_at=completed, checkpoint_number=checkpoint_number,
            previous_snapshot=previous, source_status=source.get("source_status"),
        )
        result.update(run_id=self.run_id, deployment=source.get("deployment"),
                      monitor_period_end_at=end.isoformat(), ledger_cutoff_at=cutoff.isoformat(),
                      collection_started_at=at.isoformat(), collection_completed_at=completed.isoformat(),
                      model_calls=0, telegram_messages=0, no_trade_authority=True)
        return result

    def once(self, at=None, since=None):
        at = clock(at)
        plan = self._plan() or {"started_at": clock(since or at).isoformat()}
        rows = self._records()
        result = self._capture(plan, at, None, rows[-1] if rows else None, timeout=4.2)
        result["record_kind"] = "on_demand"
        return result

    def tick(self, at=None):
        """Capture only a due checkpoint. Never replay missed observations as history."""
        if not self.enabled:
            return []
        at = clock(at)
        self.root.mkdir(parents=True, exist_ok=True, mode=0o700)
        os.chmod(self.root, 0o700)
        self.log_path.parent.mkdir(exist_ok=True, mode=0o700)
        with open(self.root / "schedule.lock", "a") as lock:
            try:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                return []
            plan = self._plan()
            if plan is None:
                plan = {"schema_version": 1, "trace": "№18,393", "run_id": self.run_id,
                        "started_at": at.isoformat(), "hours": self.hours,
                        "interval_hours": self.interval, "checkpoints": self.count,
                        "offsets_hours": [i * self.interval for i in range(self.count)],
                        "completion_observation_separate_from_periodic_checkpoints": True}
                atomic(self.plan_path, encoded(plan))
            start = clock(plan["started_at"])
            end = start + dt.timedelta(hours=self.hours)
            rows = self._records()
            emitted = []
            while len(rows) < self.count:
                index = len(rows)
                due = start + dt.timedelta(hours=index * self.interval)
                if at < due:
                    break
                if at >= due + dt.timedelta(hours=self.interval):
                    result = {"trace": "№18,393", "checkpoint_number": index + 1,
                              "run_id": self.run_id, "status": "missed_while_monitor_unavailable",
                              "timestamp_sgt": None, "observed_at": None,
                              "notable_events_since_last_checkpoint": [
                                  "No observation exists for this interval; no historical reading was fabricated."
                              ], "model_calls": 0, "no_trade_authority": True}
                else:
                    result = self._capture(plan, at, index + 1, rows[-1] if rows else None)
                result.update(record_kind="periodic_checkpoint", scheduled_at=due.isoformat(),
                              schedule_lateness_seconds=max(0, (at - due).total_seconds()))
                rows.append(result)
                atomic(self.log_path, b"".join(encoded(row) + b"\n" for row in rows))
                emitted.append(result)
            if at >= end and not self.completion_path.exists():
                result = self._capture(plan, at, None, rows[-1] if rows else None)
                result.update(record_kind="completion", completed_at=clock().isoformat(),
                              scheduled_end_at=end.isoformat(),
                              completion_lateness_seconds=(at - end).total_seconds())
                atomic(self.completion_path, encoded(result))
                emitted.append(result)
            return emitted

    def status(self):
        """Public metadata only. Never include account telemetry or credentials."""
        try:
            plan = self._plan()
            rows = self._records()
            complete = self.completion_path.exists()
            next_due = None
            if plan and not complete:
                next_due = (clock(plan["started_at"]) + dt.timedelta(
                    hours=min(len(rows) * self.interval, self.hours))).isoformat()
            return {"enabled": self.enabled, "run_id": self.run_id,
                    "state": "disabled" if not self.enabled else "completed" if complete else
                    "active" if plan else "awaiting_start", "periodic_checkpoints": len(rows),
                    "periodic_checkpoints_planned": self.count,
                    "missed_checkpoints": sum(r.get("status") == "missed_while_monitor_unavailable" for r in rows),
                    "started_at": plan.get("started_at") if plan else None, "next_due_at": next_due,
                    "last_error_type": self.last_error, "model_calls": 0,
                    "telegram_messages": 0, "no_trade_authority": True}
        except (OSError, ValueError, TypeError, KeyError) as error:
            return {"enabled": self.enabled, "state": "evidence_unavailable",
                    "last_error_type": type(error).__name__, "model_calls": 0}

    def owner_response(self, path, headers):
        route = urlsplit(path).path
        allowed = {"/owner/live-monitor", "/owner/live-monitor.jsonl", "/owner/live-monitor/once"}
        if route not in allowed:
            return None
        password = self.env.get("JIAOYI_UI_PASSWORD", "")
        if not password:
            return 503, "application/json", encoded({"error": "owner_access_not_configured"})
        supplied = headers.get("Authorization", "")
        candidate = ""
        if supplied.startswith("Bearer "):
            candidate = supplied[7:]
        elif supplied.startswith("Basic "):
            try:
                value = base64.b64decode(supplied[6:], validate=True).decode()
                user, candidate = value.split(":", 1)
                if user != "owner":
                    candidate = ""
            except (ValueError, UnicodeError):
                candidate = ""
        if not candidate or not hmac.compare_digest(hashlib.sha256(candidate.encode()).digest(),
                                                     hashlib.sha256(password.encode()).digest()):
            return 401, "application/json", encoded({"error": "owner_authentication_required"})
        try:
            if route == "/owner/live-monitor/once":
                return 200, "application/json", encoded(self.once())
            rows = self._records()
            if route.endswith(".jsonl"):
                return 200, "application/x-ndjson", b"".join(encoded(row) + b"\n" for row in rows)
            latest = json.loads(self.completion_path.read_text()) if self.completion_path.exists() else rows[-1] if rows else None
            return 200, "application/json", encoded({"monitor": self.status(), "latest": latest})
        except (OSError, ValueError, TypeError, KeyError):
            return 503, "application/json", encoded({"error": "monitor_evidence_unavailable"})

    def run(self, shutdown):
        if not self.enabled:
            return
        while not shutdown.is_set():
            try:
                for snapshot in self.tick():
                    print(encoded(snapshot).decode(), flush=True)
                self.last_error = None
            except Exception as error:
                self.last_error = type(error).__name__
            status = self.status()
            if status.get("state") == "completed":
                return
            # Check a persisted deadline, not market data, between checkpoints.
            # Event.wait remains immediately interruptible by worker shutdown.
            delay = 30.0 if self.last_error else 60.0
            if status.get("next_due_at"):
                delay = min(delay, max(1.0, (clock(status["next_due_at"]) - clock()).total_seconds()))
            shutdown.wait(delay)


def main(argv=None):
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--once", action="store_true")
    mode.add_argument("--loop-hours", type=float)
    parser.add_argument("--interval-hours", type=float, default=2)
    parser.add_argument("--state-dir", type=Path, default=Path(".live-monitor"))
    parser.add_argument("--run-id", default="18393-20261109")
    parser.add_argument("--since", help="Aware ISO timestamp for a standalone on-demand ledger")
    args = parser.parse_args(argv)
    env = dict(os.environ, LIVE_MONITOR_ENABLED="1", LIVE_MONITOR_RUN_ID=args.run_id,
               LIVE_MONITOR_HOURS=str(args.loop_hours or 24),
               LIVE_MONITOR_INTERVAL_HOURS=str(args.interval_hours))
    monitor = Monitor(env, args.state_dir)
    if args.once:
        print(encoded(monitor.once(since=args.since)).decode())
    else:
        stop = threading.Event()
        try:
            monitor.run(stop)
        except KeyboardInterrupt:
            stop.set()


if __name__ == "__main__":
    main()
