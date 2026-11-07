#!/usr/bin/env bash
# Owner-selected architecture record, №18,334. Documentation only; no broker calls.
# Default: prepare, PR, wait for explicit green CI, merge, verify main, print receipt.
# --prepare-only: write reviewable artifacts without committing or publishing.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."
exec python3 - "$@" <<'PY'
import argparse
import hashlib
import json
import re
import subprocess
import tempfile
import time
from datetime import datetime
from pathlib import Path
from zoneinfo import ZoneInfo

REPO = "ang-kl/jiaoyi"
BRANCH = "lock/s3-two-stage-architecture-18334"
BASELINE = "d551a50fed74ae8d44a845344159ea9c30d75b98"
SOURCE = Path("docs/offline-repaired-engine-results-18331.json")
SOURCE_SHA = "ca7fcbfbeee34f96f319345021037c3ef3651d52d47487f854fd9895779f0151"
SPEC = Path("docs/s3_two_stage_architecture_lock_18334.json")
REPORT = Path("doc/S3_TWO_STAGE_ARCHITECTURE_LOCK-18334-2026-11-07.md")
LOG = Path("log/CODEX_Log.txt")
SCRIPT = Path("ops/research/run_lock_s3_and_expand_peers.sh")
FILES = [SPEC, REPORT, SCRIPT, LOG]
REQUIRED = {"check", "docker", "engineering"}


def command(*args):
    return subprocess.check_output(args, text=True, timeout=120).strip()


def gh(*args):
    return json.loads(command("gh", *args))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sgt():
    return datetime.now(ZoneInfo("Asia/Singapore")).strftime("%d-%m-%Y %H:%M:%S SGT (+08:00)")


def ci_state(checks):
    """Missing, queued, skipped and null CheckRun conclusions never mean green."""
    failed = {"FAILURE", "ERROR", "CANCELLED", "TIMED_OUT", "ACTION_REQUIRED", "STALE", "STARTUP_FAILURE"}
    for check in checks:
        end = check.get("conclusion") if "status" in check else check.get("state")
        if end in failed:
            return "failed"
    required = [c for c in checks if c.get("name", c.get("context")) in REQUIRED]
    if len(required) != len(REQUIRED) or {c.get("name", c.get("context")) for c in required} != REQUIRED:
        return "pending"
    for check in checks:
        end = check.get("conclusion") if "status" in check else check.get("state")
        if "status" in check and check["status"] != "COMPLETED":
            return "pending"
        if end != "SUCCESS":
            return "failed" if end in {"SKIPPED", "NEUTRAL"} else "pending"
    return "success"


def prepare():
    # Check tracked edits before switching branches; untracked private caches stay private.
    changed = set(command("git", "diff", "--name-only", "HEAD").splitlines())
    current = command("git", "branch", "--show-current")
    if changed and (current != BRANCH or not changed <= {str(p) for p in FILES}):
        raise RuntimeError("Unrelated tracked edits present; use a clean task worktree")
    command("git", "fetch", "origin", "main")
    if current != BRANCH:
        if changed:
            raise RuntimeError("Commit or preserve tracked edits before switching branches")
        command("git", "switch", "-c", BRANCH, "origin/main")
    if subprocess.run(["git", "merge-base", "--is-ancestor", "origin/main", "HEAD"]).returncode:
        raise RuntimeError("Task branch needs reconciliation with main before publication")
    if sha(SOURCE) != SOURCE_SHA:
        raise RuntimeError("Evidence changed: review the new evidence instead of recycling this lock")
    if hashlib.sha256(subprocess.check_output(["git", "show", BASELINE + ":" + str(SOURCE)])).hexdigest() != SOURCE_SHA:
        raise RuntimeError("Baseline evidence hash mismatch")
    data = json.loads(SOURCE.read_text())
    scores = data["summary"]["scores"]["overall"]
    measured = scores["s3_two_stage"]["24"]
    assessment = data["post_run_assessment"]["s3_two_stage"]
    grok_serials = re.findall(r"№\s*([\d,]+)", Path("log/GROK_Log.txt").read_text())
    latest_grok = max((int(n.replace(",", "")) for n in grok_serials), default=0)
    if latest_grok >= 18334:
        raise RuntimeError("Grok journal advanced; reconcile the shared serial before publication")
    now = json.loads(SPEC.read_text())["locked_at_sgt"] if SPEC.exists() else sgt()
    spec = {
        "trace": "№18,334 §§1–5", "locked_at_sgt": now,
        "by": "Codex (GPT-6), effort not exposed",
        "status": "owner_selected_mainline_design_locked",
        "scope": "architecture decision on main; offline implementation already present; no live activation",
        "model": "s3_two_stage", "production_strategy_changed": False,
        "financial_pass": False, "validation_opened": False,
        "invariants": {
            "volatility_expansion_gate_bps": 48.0,
            "gate_rule": "R_bps >= 48.0 AND R_price > C_price at decision and dispatch",
            "tested_cost_rule": "C_price = median_spread + round_trip_commission + 2 * 24/10000 * reference_price; enter only when C_price < R_price",
            "interpretation": "48 bps is the owner-selected minimum, not a replacement for spread, commission or strict cost admission",
            "fixed_48_bps_floor_alone_validated": False,
            "initial_stop_r": 1.0, "breakeven_trigger_r": 1.0,
            "breakeven_lock_level_r": 0.0, "profit_target_r": 1.38,
            "breakeven_basis": "gross entry-price stop; costs and gaps can make the exit net-negative",
            "stop_amendment": "executable quote triggers; effective only on a strictly later valid quote; original bracket has priority",
            "target_basis": "gross whole-position target, rounded outward to price grid",
            "primary_cost_reserve_bps_per_side": 24.0,
            "cost_calibration": "research assumption; not an empirically calibrated broker estimate",
            "goal": "73% net wins and reference risk:reward 1:1.38 over latest 20 closed trades for each strategy/account; 15/20 = 75%"
        },
        "authorized_peer_pairs": {
            "energy": {"primary": "SpotCrude", "peer": "SpotBrent"},
            "equity_index": {"primary": "US500", "peer": "NAS100"},
            "metal": {"primary": "XAUUSD", "peer": "XAGUSD"}
        },
        "peer_status": {
            "existing_offline_mapping": True, "new_peer_data_collected": False,
            "missing_from_frozen_corpus": ["NAS100", "XAGUSD"],
            "scope": "authorised macro research peer confirmation; no new production cross-asset filter"
        },
        "empirical_observation": {
            "source_file": str(SOURCE), "source_sha256": SOURCE_SHA,
            "pr": 183, "pr_commit": BASELINE,
            **{k: measured[k] for k in ["closed_trades", "wins", "win_rate", "mean_net_r", "profit_factor"]},
            "profit_factor_basis": "normalised R, not pooled account dollars",
            "protocol_flagged_closed_trades": assessment["protocol_flagged_closed_trades"],
            "protocol_flagged_wins": assessment["protocol_flagged_wins"],
            "without_this_protocol_flag": assessment["without_this_protocol_flag"],
            "source_protocol_compliance": data["source_protocol_compliance"],
            "qualification": "exploratory development result; not validation or proof of a consistent edge"
        },
        "next_stage": {
            "status": "planned_not_run", "validation_holdout": "2025-08-19 through 2025-09-05 remains sealed",
            "sampling": "freeze representative dates, symbol coverage, costs, cohort/job budget and stopping rule before observing new outcomes",
            "minimum_closed_trades_milestone": 20,
            "milestone_caveat": "20 pooled research closes do not establish the per-strategy/account goal; no outcome-driven sampling until enough wins",
            "source_control": "exclude spent intervals from new evaluation, retain failures/censoring, do not recollect under new job names",
            "new_broker_requests": 0
        }
    }
    SPEC.parent.mkdir(parents=True, exist_ok=True)
    SPEC.write_text(json.dumps(spec, indent=2, ensure_ascii=False) + "\n")
    dynamic = scores["s3_dynamic_gate"]["24"]
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(f"""# Strategy 3 two-stage architecture decision and peer authorisation

№18,334 §§1–5 · {now} · by Codex (GPT-6), effort not exposed.

## §1. Decision and evidence

**The owner selects `s3_two_stage` as the mainline design direction and confirms the three peer pairs below. This PR records that decision. It does not activate a production strategy or establish financial qualification.** The offline implementation is already merged in [PR #183](https://github.com/{REPO}/pull/183), commit `{BASELINE}` (not PR #182).

At the assumed 24 bp per side, the existing five oil closes have {measured['wins']}/{measured['closed_trades']} net wins ({measured['win_rate']:.1%}), mean {measured['mean_net_r']:+.8f}R and PF(R) {measured['profit_factor']:.8f}. Three closes carry inherited source-protocol flags; two are winners that were censored in the baseline. The two closes without that particular flag average {assessment['without_this_protocol_flag']['mean_net_r']:+.8f}R and retain other quote-quality limitations. Index/gold have no closed trades in this arm. These reused development dates are not clean validation.

The dynamic gate arm produced {dynamic['wins']}/{dynamic['closed_trades']} wins ({dynamic['losses']} losses), mean {dynamic['mean_net_r']:+.6f}R. This rejects that cost-gate replacement under the tested assumptions. It does not prove a universal 48 bp cutoff or identify volatility alone as the cause.

## §2. Locked interpretation of admission and exits

- Owner-selected minimum: `R_bps >= 48.0`, where `R_bps = 10000 * R_price / reference_price`.
- Retain the stricter **tested** admission: `R_price > median_spread + round_trip_commission + 2 * 24/10000 * reference_price`, checked at decision and dispatch. A bare `R_bps >= 48` drops spread/commission and permits equality; it is not equivalent and is not what produced the five-trade result. With non-negative costs at 24 bp/side, the tested rule already implies the minimum.
- Original stop at −1R; executable bid for longs / ask for shorts reaching +1R proposes an entry-price stop. It becomes effective on a strictly later valid quote; existing stop/target processing retains priority. Gaps use executable quotes. Whole-position gross target +1.38R, rounded outward; no extra trailing or partial exit in this arm. Native session deadlines and censored source boundaries remain.
- Entry-price breakeven is **gross**, not net. The experiment changes both trailing and target geometry. It does not isolate a trailing-only effect.
- There is **no guaranteed positive expectancy above 45% wins**. For the simplified frictionless binary +1.38R/−1R model, the threshold is `1/2.38 ≈ 42.02%`; with constant cost `c` in R it becomes `(1+c)/2.38`. At 45%, gross expectancy is just +0.071R before costs. Actual two-stage exits, flat exits and gaps require their full realised outcome distribution.

## §3. Authorised peers and actual availability

| Class | Primary | Peer | Frozen data / implication |
| --- | --- | --- | --- |
| Energy | SpotCrude | SpotBrent | Both present; existing offline mapping |
| Equity index | US500 | NAS100 | NAS100 absent; index macro confirmation unavailable |
| Metal | XAUUSD | XAGUSD | XAGUSD absent; gold macro confirmation unavailable |

These are owner-authorised research relationships, not proven predictive signals. Mappings already exist in PR #183's macro arm; this record does not add a peer filter to live S3 or collect missing observations. No fabricated peer bars or fallback substitution.

## §4. Next evaluation and goal tracking

Validation **19 August–5 September 2025 remains sealed**. Next work is a separately frozen, bounded multi-cohort study: predeclare representative dates, session coverage, actual peer availability, costs, cohort/job budget, failure handling and stopping rule before outcomes. Respect the spent-source registry; no duplicate reads disguised as new jobs. Preserve all attempted jobs, nonentries, failures and censoring.

Twenty closed trades is an interim sample milestone, not a qualification rule or permission to keep sampling until favourable. Report energy/index/metal separately and distinguish simulated research from account outcomes. The project goal remains **73% net wins / reference risk:reward 1:1.38 over the latest 20 closes per strategy per account** (at least 15 wins gives 75%). Qualified progress from this documentation run: **none demonstrated**; no new trades or market replays were run. No live promotion follows from the pooled 60% observation.

## §5. Traceability and release checks

The [JSON specification](../{SPEC}) retains the actual measured values and limitations. Evidence: [{SOURCE.name}](../{SOURCE}), SHA256 `{SOURCE_SHA}`; [complete experiment report](OFFLINE_REPAIRED_ENGINE_REPORT-18331-2026-11-07.md). The source audit remains FAILED, affecting 20 episodes overall.

The accompanying script creates a task branch before artifact writes, stages only four named files, requires explicit success from `check`, `docker` and `engineering` on the unchanged PR head, and verifies merged file hashes. Missing/pending/null/failed checks cannot merge. The final review block is generated from the actual merge response. Timestamps are actual SGT recording times; journals preserve serial/paragraph/model references and consult GROK_Log.txt. No Rust runtime, broker setting, research results or deployment configuration is modified by this record.
""")
    log_text = LOG.read_text()
    if "№18,334 §§1–5" not in log_text:
        number = max(map(int, re.findall(r"\[§LOG\.(\d+)\]", log_text)), default=0) + 1
        LOG.write_text(log_text + f"\n[§LOG.{number}] №18,334 §§1–5 · {now}; by <Codex (GPT-6), effort: not exposed>.\nOwner-selected s3_two_stage mainline design and Crude/Brent, US500/NAS100, XAUUSD/XAGUSD peer authorisation recorded.\n48 bp minimum retains strict spread+commission+two-sided reserve < R; floor alone is not the tested rule.\nMeasured PR #183 result remains 3/5 wins, +0.12385077R, PF1.45149070; 3 closes protocol-flagged.\nRemoved positive-expectancy guarantee; gross entry-price stop does not guarantee net breakeven.\nNo production activation, new broker reads, validation opening or financial qualification.\nChecked GROK_Log.txt before choosing serial; latest recorded Grok serial remains №{latest_grok:,}.\nPublication requires explicit head-matched CI success and merged artifact verification.\n")
    return spec


def publish(spec):
    command("bash", "-n", str(SCRIPT))
    command("git", "diff", "--check")
    paths = [str(p) for p in FILES]
    staged = set(command("git", "diff", "--cached", "--name-only").splitlines())
    if not staged <= set(paths):
        raise RuntimeError("Unrelated staged files; refusing to commit them")
    command("git", "add", "--", *paths)
    if subprocess.run(["git", "diff", "--cached", "--quiet"]).returncode:
        command("git", "commit", "-m", "docs: record S3 two-stage architecture and peer authorisation (№18,334 §§1–5)")
    head = command("git", "rev-parse", "HEAD")
    command("git", "-c", "credential.helper=", "-c", "credential.helper=!gh auth git-credential", "push", "-u", "origin", BRANCH)
    hashes = {str(p): sha(p) for p in FILES}
    with tempfile.TemporaryDirectory(prefix="jiaoyi-18334-") as temp:
        body = Path(temp) / "body.md"
        body.write_text(f"№18,334 §§1–5 · {spec['locked_at_sgt']} · by Codex (GPT-6), effort not exposed.\n\nRecords the owner's s3_two_stage design choice and three authorised peer pairs. The supplied script conflated five exploratory trades with validation, omitted spread/commission from its claimed equivalent gate, and could mistake unfinished CI for green. This version preserves the 48 bp minimum alongside the strict tested cost screen, discloses inherited protocol flags and missing peers, and removes the unsupported expectancy guarantee.\n\nOnly the architecture JSON, report, journal and publication script change. No production strategy activation or new market-data collection. The 73% / 1:1.38 latest-20 goal remains unproven.\n\nValidation: shell syntax, JSON/evidence consistency and CI-state cases; required check/docker/engineering must all succeed on this exact head before merge. Report: `{REPORT}`.\n")
        prs = gh("pr", "list", "--repo", REPO, "--head", BRANCH, "--state", "open", "--json", "url,number")
        if len(prs) > 1:
            raise RuntimeError("Ambiguous open PRs")
        url = prs[0]["url"] if prs else command("gh", "pr", "create", "--repo", REPO, "--base", "main", "--head", BRANCH, "--title", "Docs: S3 two-stage design lock and peer authorisation (№18,334 §§1–5)", "--body-file", str(body))
        number = url.rsplit("/", 1)[-1]
        print("ARCHITECTURE_PR " + url, flush=True)
        deadline = time.monotonic() + 1800
        while True:
            pr = gh("pr", "view", number, "--repo", REPO, "--json", "state,headRefOid,statusCheckRollup")
            if pr["state"] != "OPEN" or pr["headRefOid"] != head:
                raise RuntimeError("PR state/head changed; inspect before proceeding")
            state = ci_state(pr["statusCheckRollup"])
            print("CI " + state, flush=True)
            if state == "failed":
                raise RuntimeError("CI failed or was skipped; inspect checks before proceeding")
            if state == "success":
                break
            if time.monotonic() > deadline:
                raise RuntimeError("CI deadline exceeded; no merge")
            time.sleep(15)
        command("gh", "pr", "merge", number, "--repo", REPO, "--squash", "--match-head-commit", head)
        merged = gh("pr", "view", number, "--repo", REPO, "--json", "state,mergeCommit,mergedAt")
        if merged["state"] != "MERGED":
            raise RuntimeError("Merge not confirmed")
        merge = merged["mergeCommit"]["oid"]
        command("git", "fetch", "origin", "main")
        for path, wanted in hashes.items():
            for ref in [merge, "origin/main"]:
                actual = hashlib.sha256(subprocess.check_output(["git", "show", ref + ":" + path])).hexdigest()
                if actual != wanted:
                    raise RuntimeError("Merged artifact mismatch: " + path)
        block = {
            "trace": "№18,334 §§1–5", "verified_at_sgt": sgt(), "by": spec["by"],
            "status": spec["status"], "pr_url": url, "pr_merge_commit": merge,
            "locked_model": spec["model"], "invariants": spec["invariants"],
            "peers_authorized": ["US500/NAS100", "XAUUSD/XAGUSD", "SpotCrude/SpotBrent"],
            "missing_peer_data": spec["peer_status"]["missing_from_frozen_corpus"],
            "financial_pass": False, "production_strategy_changed": False,
            "validation_opened": False, "new_broker_requests": 0, "ci": "check/docker/engineering SUCCESS",
            "report_file": str(REPORT), "artifact_sha256": hashes
        }
        receipt = Path(temp) / "receipt.json"
        receipt.write_text(json.dumps({"body": body.read_text() + "\nMerged and verified on main:\n```json\n" + json.dumps(block, indent=2, ensure_ascii=False) + "\n```\n"}))
        command("gh", "api", "--method", "PATCH", f"repos/{REPO}/pulls/{number}", "--input", str(receipt), "--jq", ".html_url")
        print("=== COPY-PASTE FOR GEMINI REVIEW ===\n" + json.dumps(block, indent=2, ensure_ascii=False) + "\n====================================", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Record the №18,334 owner-selected design; no live trading changes")
    parser.add_argument("--prepare-only", action="store_true")
    args = parser.parse_args()
    specification = prepare()
    print("Prepared four architecture-record artifacts; no new research or live activation", flush=True)
    if not args.prepare_only:
        publish(specification)
PY
