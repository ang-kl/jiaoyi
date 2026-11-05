#!/usr/bin/env python3
"""Collect one completed registered four-hour window for each research symbol.

Recent collection only: no replay, model API, order or automatic trading action.
"""
import argparse
import datetime as dt
import hashlib
import json
import os
import time
import urllib.request
from pathlib import Path
from urllib.parse import urlsplit

import broker_backtest
import class_research


def call(base, password, request=None):
    headers = {"Authorization": "Bearer " + password, "Cache-Control": "no-cache"}
    data = None
    if request is not None:
        headers["Content-Type"] = "application/json"
        data = json.dumps(request).encode()
    req = urllib.request.Request(base + "/owner/market-data", data=data, headers=headers)
    with urllib.request.build_opener(broker_backtest.NoRedirect).open(req, timeout=25) as response:
        raw = response.read(32_000_001)
    if len(raw) > 32_000_000:
        raise RuntimeError("collection response exceeds32MB")
    return json.loads(raw)


def source_digest(source):
    # Native s3_ticks::Source field order, with integer ticks/contract fields.
    canonical = dict(bars=source["bars"],
                     bids=[dict(ts_ms=t["ts_ms"], price_e5=t["price_e5"]) for t in source["bids"]],
                     asks=[dict(ts_ms=t["ts_ms"], price_e5=t["price_e5"]) for t in source["asks"]],
                     contract={k: source["contract"][k] for k in ("digits", "min_volume", "step_volume")})
    if canonical["bars"]:
        raise ValueError("collection unexpectedly contains bars")
    return hashlib.sha256(json.dumps(canonical, separators=(",", ":"), allow_nan=False).encode()).hexdigest()


def validate(job, req):
    if job.get("kind") != "recent_market_data" or job.get("request") != req:
        raise ValueError("collection request/kind mismatch")
    if job["status"] == "complete":
        result = job["result"]
        if result["qualification"] != "RECENT_COLLECTION_ONLY_NOT_FINANCIAL_PASS" or result["study"] is not None:
            raise ValueError("collection unexpectedly ran a study")
        if source_digest(result["input"]) != result["input_sha256"]:
            raise ValueError("source digest mismatch")


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--date", type=dt.date.fromisoformat, required=True, help="local registered session date")
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--account", type=int, default=49236051)
    p.add_argument("--base", default="https://jiaoyi.up.railway.app")
    a = p.parse_args()
    url = urlsplit(a.base)
    if url.scheme != "https" or url.username or url.password or url.path not in ("", "/") or url.query or url.fragment:
        raise SystemExit("HTTPS origin without credentials or path required")
    plan = json.loads(class_research.PLAN.read_text())
    plan["development_dates"] = [a.date.isoformat()]
    requests = []
    for cls, symbols in plan["classes"].items():
        for symbol in symbols:
            _, start, end = next(class_research.windows(plan, "development", symbol))
            first = dt.datetime.fromisoformat(start.replace("Z", "+00:00"))
            last = dt.datetime.fromisoformat(end.replace("Z", "+00:00"))
            if first < dt.datetime(2026, 10, 24, tzinfo=dt.timezone.utc) or last + dt.timedelta(seconds=5) > dt.datetime.now(dt.timezone.utc):
                raise SystemExit("all windows including five-second tails must be closed and post-holdout")
            requests.append((cls, dict(account=a.account, symbol=symbol, **{"from": start, "to": end})))
    a.output.mkdir(parents=True, exist_ok=True)
    manifest_path = a.output / "collection-plan.json"
    frozen = dict(date=a.date.isoformat(), account=a.account, requests=requests,
                  scope="recent collection only; not a fresh holdout or financial qualification",
                  limits=dict(jobs=20, ticks_per_job=200000, pages_per_job=120, timeout_seconds=240))
    if manifest_path.exists() and json.loads(manifest_path.read_text()) != json.loads(json.dumps(frozen)):
        raise SystemExit("collection plan differs; use a separate directory")
    manifest_path.write_text(json.dumps(frozen, indent=2))
    password = os.environ["JIAOYI_UI_PASSWORD"]
    summary = []
    for cls, req in requests:
        path = a.output / (req["symbol"] + ".json")
        if path.exists():
            job = json.loads(path.read_text())
            validate(job, req)
            if job["status"] == "running":
                raise RuntimeError("saved unfinished job; inspect before retry")
        else:
            current = call(a.base.rstrip("/"), password)
            if current and current["status"] == "running":
                raise RuntimeError("another job is active; not replaced")
            receipt = call(a.base.rstrip("/"), password, req)
            identity = receipt["id"]
            deadline = time.monotonic() + 275
            print("submitted", req["symbol"], identity, flush=True)
            while time.monotonic() < deadline:
                time.sleep(3)
                job = call(a.base.rstrip("/"), password)
                if not job or job["id"] != identity:
                    raise RuntimeError("job replaced or service restarted; inspect before retry")
                if job["status"] != "running":
                    break
            else:
                raise RuntimeError("poll deadline; inspect before retry")
            validate(job, req)
            path.write_text(json.dumps(job, separators=(",", ":")))
            path.chmod(0o600)
        result = job.get("result") or {}
        summary.append(dict(symbol=req["symbol"], asset_class=cls, request=req,
                            status=job["status"], error=job.get("error"),
                            coverage=result.get("coverage"), tick_pages=result.get("tick_pages"),
                            input_sha256=result.get("input_sha256"),
                            file_sha256=hashlib.sha256(path.read_bytes()).hexdigest(), file=path.name))
        (a.output / "collection-summary.json").write_text(json.dumps(dict(
            generated_at_utc=dt.datetime.now(dt.timezone.utc).isoformat(), expected_jobs=20,
            completed_jobs=sum(r["status"] == "complete" for r in summary),
            both_sides_present=sum(bool(r["coverage"] and r["coverage"]["both_sides_present"]) for r in summary),
            financial_pass=False, rows=summary), indent=2))
        print("finished", req["symbol"], job["status"], result.get("coverage"), flush=True)


if __name__ == "__main__":
    main()
