"""Deterministic HTML files and receipt-backed owner-only Telegram delivery."""
import datetime as dt
import hashlib
import html
import json
import re
from pathlib import Path
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid
from zoneinfo import ZoneInfo

from stop_loss_reviews import PREFIX as INTERNAL_REVIEW_PREFIX

SGT = ZoneInfo("Asia/Singapore")


class DeliveryError(RuntimeError):
    def __init__(self, code, uncertain=False):
        super().__init__(code)
        self.uncertain = uncertain


def stamp(value):
    if isinstance(value, (int, float)):
        value = dt.datetime.fromtimestamp(value, dt.timezone.utc)
    elif isinstance(value, str):
        value = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    return value.astimezone(SGT).strftime("%d %b %Y %H:%M SGT")


def render(data):
    esc = lambda value: html.escape(str(value), quote=True)
    cutoff = dt.datetime.fromisoformat(data["until"].replace("Z", "+00:00")).astimezone(SGT)
    serial = esc(data.get("serial", "E-" + cutoff.strftime("%Y%m%d-%H%M%S")))
    jobs = "".join(
        "<tr><td>" + esc(j["id"]) + "</td><td>" + esc(j["state"]) + "</td><td>"
        + esc(j.get("summary", "No preserved model summary")) + "</td><td>"
        + esc(j.get("diagnostic", "")) + "</td></tr>" for j in data.get("jobs", []))
    commits = "".join("<li><code>" + esc(c["sha"]) + "</code> " + esc(c["subject"]) + "</li>"
                      for c in data.get("commits", [])) or "<li>No merged commit observed in this window.</li>"
    notes = "".join("<li>" + esc(n) + "</li>" for n in data.get("notes", []))
    tests = "".join("<details><summary>Job " + esc(j["id"]) + " — reported validation</summary><p>"
                    + esc(j.get("tests", "No preserved test receipt")) + "</p></details>"
                    for j in data.get("jobs", []))
    document = ("<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'>"
            "<title>Jiaoyi engineering report</title><style>body{font:15px/1.55 system-ui;margin:24px auto;max-width:960px;padding:0 16px;color:#15243a}"
            "h1{font-size:24px}h2{font-size:18px;margin-top:28px}table{border-collapse:collapse;width:100%;font-size:13px}td,th{padding:9px;border-bottom:1px solid #ccd5df;text-align:left;vertical-align:top}"
            ".flow{padding:16px;background:#edf4fc;border-radius:8px}code{overflow-wrap:anywhere}details{margin:12px 0}summary{cursor:pointer}p,li{overflow-wrap:anywhere}</style>"
            "<h1>№" + serial + " · Jiaoyi engineering change report</h1><p>Generated "
            + esc(stamp(dt.datetime.now(dt.timezone.utc))) + ". Evidence window: " + esc(stamp(data["since"])) + " → "
            + esc(stamp(data["until"])) + ". Evidence cutoffs are stated; attempts are not releases.</p>"
            "<p><b>Worker:</b> " + esc(data.get("state", "unknown")) + " · <b>SGT attempts:</b> "
            + esc(data.get("sessions", "unknown")) + "/" + esc(data.get("limit", "unknown")) + "</p>"
            "<p><b>Invariant benchmark:</b> each strategy/account's latest20 fully closed owned positions; "
            "≥73% net wins (15/20=75%) and the risk:reward objective1:1.38. Realized net payoff reference1.38 is measured separately from registered stop/target geometry. "
            "Fewer than20 is insufficient evidence; one passing overlapping window is not consistency.</p>"
            "<h2>Delivered changes on main</h2><ul>" + commits + "</ul>"
            "<h2>Worker attempts and unreleased candidates</h2><table><tr><th>Job</th><th>State</th><th>Candidate/result</th><th>Release diagnostic</th></tr>"
            + jobs + "</table>" + tests + "<h2>Changes added in this request / pending facts</h2><ul>" + notes + "</ul>"
            "<h2>AI engineering: how this works</h2><div class=flow>Persistent coordinator + SQLite queue → unprivileged Codex main agent → bounded sub-agents → scope checks + Git PR → exact-head CI → merge → Railway revision + behavior check</div>"
            "<p>The coordinator schedules work and holds release credentials. Model processes receive no GitHub, Railway, Telegram or broker credentials. "
            "Sub-agents need a persisted parent rollout; ephemeral execution broke that context. Explicit acceptance evidence now prevents a partial check from unlocking work. "
            "A proposed patch, a passing model-reported test, an accepted PR and a verified deployment are separate milestones.</p>"
            "<p>The allowance is a ceiling, not a quota. Unchanged material evidence skips automatic model work; "
            "new trade cohorts or material health/goal changes can trigger assessment. Unresolved retained proposals block fresh model attempts while reporting and CI polling continue.</p>"
            "<p>Reporting uses deterministic templates and a durable delivery outbox, independently of model work and trading exits. "
            "Only Telegram's positive message receipt marks a file delivered. A lost acknowledgement is held for review, rather than blindly resending a possible duplicate.</p>"
            "<h2>Token and hosting cost</h2><p>18 versus 4 allows 4.5× as many top-level attempts; it does not predict token usage. "
            "Retries, prompt size, reasoning and sub-agents affect consumption. The limit counts coordinator attempts, not every child call or token, and is not a money ceiling. "
            "ChatGPT/Codex plan authentication remains in use; no separately billed API route was added. This HTML renderer and Telegram delivery make zero LLM calls. "
            "Railway compute/storage costs remain separate. Failed attempts still consume allowance.</p>"
            "<p><small>Unknown evidence is not zero. No report changes entries, existing pre-orders, protective stops, sizing, account activation or owner-only decisions. "
            "Financial success and consistent latest-20 targets remain unproved.</small></p></html>")
    number = 0
    def paragraph(match):
        nonlocal number
        number += 1
        return match.group(0) + "[§" + serial + "." + str(number) + "] "
    return re.sub(r"<p>", paragraph, document)


def send_document(token, chat_id, filename, body, caption):
    if not token or not chat_id:
        raise DeliveryError("telegram_not_configured")
    boundary = "jiaoyi-" + uuid.uuid4().hex
    content = body.encode("utf-8")
    if len(content) > 5 * 1024 * 1024:
        raise DeliveryError("html_too_large")
    filename = Path(filename).name.replace('"', "").replace("\r", "").replace("\n", "")
    parts = []
    for name, value in (("chat_id", chat_id), ("caption", caption[:1024])):
        parts.append((f'--{boundary}\r\nContent-Disposition: form-data; name="{name}"\r\n\r\n{value}\r\n').encode())
    parts += [(f'--{boundary}\r\nContent-Disposition: form-data; name="document"; filename="{filename}"\r\nContent-Type: text/html; charset=utf-8\r\n\r\n').encode(), content,
              (f"\r\n--{boundary}--\r\n").encode()]
    request = urllib.request.Request("https://api.telegram.org/bot" + token + "/sendDocument",
        data=b"".join(parts), headers={"Content-Type": "multipart/form-data; boundary=" + boundary,
                                     "User-Agent": "jiaoyi-reports/1.0"}, method="POST")
    try:
        with urllib.request.urlopen(request, timeout=25) as response:
            result = json.loads(response.read(1024 * 1024))
    except urllib.error.HTTPError as error:
        raise DeliveryError("telegram_http_" + str(error.code), uncertain=error.code >= 500) from None
    except Exception:
        # A lost acknowledgement can follow a successful POST: do not replay blindly.
        raise DeliveryError("telegram_response_unknown", uncertain=True) from None
    if not isinstance(result, dict):
        raise DeliveryError("telegram_receipt_unknown", uncertain=True)
    receipt = result.get("result", {})
    if result.get("ok") is not True:
        raise DeliveryError("telegram_rejected")
    if not isinstance(receipt, dict) or not isinstance(receipt.get("message_id"), int):
        raise DeliveryError("telegram_receipt_unknown", uncertain=True)
    return receipt["message_id"]


def send_message(token, chat_id, text):
    """One short owner alert. No document, no retry storm."""
    if not token or not chat_id:
        raise DeliveryError("telegram_not_configured")
    body = urllib.parse.urlencode({
        "chat_id": chat_id,
        "text": text[:3500],
        "disable_web_page_preview": "true",
    }).encode()
    request = urllib.request.Request(
        "https://api.telegram.org/bot" + token + "/sendMessage",
        data=body, method="POST")
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            payload = json.loads(response.read().decode())
    except urllib.error.HTTPError as error:
        raise DeliveryError("telegram_http_" + str(error.code), uncertain=error.code >= 500) from None
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError):
        raise DeliveryError("telegram_response_unknown", uncertain=True) from None
    if not isinstance(payload, dict) or payload.get("ok") is not True:
        raise DeliveryError("telegram_rejected")
    message = (payload.get("result") or {}).get("message_id")
    if not isinstance(message, int) or message <= 0:
        raise DeliveryError("telegram_receipt_unknown", uncertain=True)
    return message


def collect(coordinator, hours=6, until=None):
    until = until or dt.datetime.now(dt.timezone.utc)
    since = until - dt.timedelta(hours=hours)
    with coordinator.store.connect() as db:
        jobs = [dict(r) for r in db.execute("SELECT * FROM jobs WHERE updated>=? AND updated<=? AND task NOT LIKE ? ORDER BY id LIMIT 100",
                                           (since.isoformat(), until.isoformat(), INTERNAL_REVIEW_PREFIX + "%"))]
        internal_review_count = db.execute(
            "SELECT count(*) FROM jobs WHERE updated>=? AND updated<=? AND task LIKE ?",
            (since.isoformat(), until.isoformat(), INTERNAL_REVIEW_PREFIX + "%")).fetchone()[0]
        for job in jobs:
            row = db.execute("SELECT detail FROM events WHERE kind='model_report' AND job=? ORDER BY id DESC LIMIT 1", (job["id"],)).fetchone()
            try:
                report = json.loads(row[0] if row else job.get("result") or "{}")
            except (ValueError, TypeError):
                report = {}
            job["summary"] = str(report.get("summary", "No preserved model summary; state is " + job["state"]))[:900]
            job["tests"] = str(report.get("tests", "No independently verified test receipt"))[:2200]
            job["diagnostic"] = ""
            if job["state"] == "failed":
                try:
                    job["diagnostic"] = json.dumps(coordinator.diagnose_job(job), ensure_ascii=False)[:1800]
                except Exception as error:
                    job["diagnostic"] = "Diagnostic unavailable: " + type(error).__name__
    commits = []
    repo = coordinator.config.root / "repository"
    if repo.exists():
        try:
            lines = coordinator.git("log", "origin/main", "--since=" + since.isoformat(), "--until=" + until.isoformat(),
                                    "--format=%h%x1f%s", "-30", cwd=repo).splitlines()
            commits = [{"sha": line.split("\x1f", 1)[0], "subject": line.split("\x1f", 1)[1]} for line in lines if "\x1f" in line]
        except Exception:
            pass
    notes = ["Model test summaries are claims until their CI/deployment receipts are verified.",
             "Telegram files use deterministic templates and confirmed delivery receipts; no extra AI inference.",
             "Internal stop-loss review jobs in this window: " + str(internal_review_count)
             + "; individual findings stay internal and are omitted from Telegram."]
    if coordinator.config.env.get("GH_TOKEN_EXPIRES_ON"):
        notes.append("Owner-reported worker GitHub token expiry date: " + coordinator.config.env["GH_TOKEN_EXPIRES_ON"]
                     + "; exact expiry instant was not supplied. Rotate before this date.")
    return {"since": since.isoformat(), "until": until.isoformat(), "state": coordinator.status()["state"],
            "sessions": coordinator.store.used(), "limit": coordinator.config.limit, "jobs": jobs, "commits": commits,
            "notes": notes, "internal_review_count": internal_review_count}


class Reporter:
    def __init__(self, coordinator):
        self.coordinator = coordinator
        self.store = coordinator.store
        self.env = coordinator.config.env
        self.interval = int(self.env.get("ENGINEERING_TELEGRAM_INTERVAL_SECONDS", "21600"))
        if not 300 <= self.interval <= 86400:
            raise ValueError("engineering report interval outside range")
        with self.store.connect() as db:
            db.execute("CREATE TABLE IF NOT EXISTS engineering_report_delivery (id INTEGER PRIMARY KEY CHECK(id=1), next_due REAL NOT NULL, state TEXT NOT NULL, path TEXT, message_id INTEGER, last_error TEXT, retry_at REAL)")
            db.execute("INSERT OR IGNORE INTO engineering_report_delivery(id,next_due,state) VALUES(1,?,'waiting')", (time.time() + self.interval,))

    def status(self):
        with self.store.connect() as db:
            row = dict(db.execute("SELECT next_due,state,message_id,last_error FROM engineering_report_delivery WHERE id=1").fetchone())
        return {"configured": bool(self.env.get("TELEGRAM_BOT_TOKEN") and self.env.get("TELEGRAM_OWNER_CHAT_ID")),
                "interval_seconds": self.interval, "delivery": row,
                "source_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}

    def tick(self):
        if not self.status()["configured"]:
            return
        with self.store.connect() as db:
            row = dict(db.execute("SELECT * FROM engineering_report_delivery WHERE id=1").fetchone())
        current = time.time()
        if row["state"] in ("sending", "uncertain") or current < row["next_due"] or current < (row["retry_at"] or 0):
            return
        reports = self.store.private / "html-reports"
        reports.mkdir(mode=0o700, exist_ok=True)
        if row["path"]:
            path = Path(row["path"])
        else:
            path = reports / ("engineering-" + str(int(row["next_due"])) + ".html")
            path.write_text(render(collect(self.coordinator)), encoding="utf-8")
            path.chmod(0o600)
        with self.store.connect() as db:
            # Persist before POST. An interrupted send is uncertain on restart.
            db.execute("UPDATE engineering_report_delivery SET state='sending',path=? WHERE id=1", (str(path),))
        try:
            message = send_document(self.env["TELEGRAM_BOT_TOKEN"], self.env["TELEGRAM_OWNER_CHAT_ID"], path.name,
                                    path.read_text(), "Jiaoyi engineering · last six hours · HTML report")
        except DeliveryError as error:
            with self.store.connect() as db:
                db.execute("UPDATE engineering_report_delivery SET state=?,last_error=?,retry_at=? WHERE id=1",
                           ("uncertain" if error.uncertain else "retry", str(error), current + 900))
            return
        with self.store.connect() as db:
            db.execute("UPDATE engineering_report_delivery SET state='waiting',next_due=?,path=NULL,message_id=?,last_error=NULL,retry_at=NULL WHERE id=1",
                       (current + self.interval, message))
        self.store.event("telegram_report_delivered", detail="message_id=" + str(message))
        # Bound saved reports independently of the model transcript retention.
        for old in sorted(reports.glob("engineering-*.html"), key=lambda p: p.stat().st_mtime)[:-40]:
            old.unlink()

    def run(self):
        # Private read-only diagnostics explain retained failed proposals without
        # paying for another model attempt or exposing a public detail endpoint.
        for job in reversed(self.store.jobs(("failed",))[-10:]):
            try:
                diagnostic = self.coordinator.diagnose_job(job)
                print(json.dumps({"event": "engineering_release_diagnostic", "job": job["id"],
                                  "diagnostic": diagnostic}), flush=True)
            except Exception as error:
                self.store.event("telegram_diagnostic_error", job["id"], type(error).__name__)
        while not self.coordinator.shutdown.is_set():
            try:
                self.tick()
            except Exception as error:
                self.store.event("telegram_report_error", detail=type(error).__name__)
            try:
                self.cost_tick()
            except Exception as error:
                self.store.event("grok_cost_alert_error", detail=type(error).__name__)
            self.coordinator.shutdown.wait(30)

    def record_usage(self, model, input_tokens, output_tokens, cached_tokens=0, now=None):
        """Add one metered call. Price the model the response returned. № 17,637."""
        from decimal import Decimal
        from api_cost import SGT, estimate, notices
        if not (self.env.get("TELEGRAM_BOT_TOKEN") and self.env.get("TELEGRAM_OWNER_CHAT_ID")):
            return
        now = now or dt.datetime.now(SGT)
        added = estimate(model, input_tokens, output_tokens, cached_tokens)
        with self.store.connect() as db:
            db.execute(
                "CREATE TABLE IF NOT EXISTS grok_api_spend ("
                "id INTEGER PRIMARY KEY CHECK(id=1), usd TEXT NOT NULL, "
                "last_daily TEXT, notified_step INTEGER NOT NULL)")
            db.execute("INSERT OR IGNORE INTO grok_api_spend(id,usd,notified_step) VALUES(1,'0',0)")
            row = dict(db.execute("SELECT usd,last_daily FROM grok_api_spend WHERE id=1").fetchone())
        before = Decimal(row["usd"])
        after = before + added
        pending = [line for line in notices(before, after, now, row["last_daily"]) if "crossed $" in line]
        with self.store.connect() as db:
            db.execute("UPDATE grok_api_spend SET usd=? WHERE id=1", (format(after, "f"),))
        if pending:
            send_message(self.env["TELEGRAM_BOT_TOKEN"], self.env["TELEGRAM_OWNER_CHAT_ID"], "\n".join(pending))
            step = int(after // Decimal("10")) * 10
            with self.store.connect() as db:
                db.execute("UPDATE grok_api_spend SET notified_step=? WHERE id=1", (step,))

    def cost_tick(self, now=None):
        """11:00 SGT daily. A $10 step is sent by record_usage. № 17,635."""
        from decimal import Decimal
        from api_cost import SGT, notices
        if not (self.env.get("TELEGRAM_BOT_TOKEN") and self.env.get("TELEGRAM_OWNER_CHAT_ID")):
            return
        now = now or dt.datetime.now(SGT)
        with self.store.connect() as db:
            db.execute(
                "CREATE TABLE IF NOT EXISTS grok_api_spend ("
                "id INTEGER PRIMARY KEY CHECK(id=1), usd TEXT NOT NULL, "
                "last_daily TEXT, notified_step INTEGER NOT NULL)")
            db.execute("INSERT OR IGNORE INTO grok_api_spend(id,usd,notified_step) VALUES(1,'0',0)")
            row = dict(db.execute("SELECT usd,last_daily,notified_step FROM grok_api_spend WHERE id=1").fetchone())
        spent = Decimal(row["usd"])
        pending = [line for line in notices(spent, spent, now, row["last_daily"]) if "daily cost" in line]
        if not pending:
            return
        send_message(self.env["TELEGRAM_BOT_TOKEN"], self.env["TELEGRAM_OWNER_CHAT_ID"], "\n".join(pending))
        with self.store.connect() as db:
            db.execute("UPDATE grok_api_spend SET last_daily=? WHERE id=1", (now.astimezone(SGT).date().isoformat(),))
