#!/usr/bin/env python3
"""Offline cost coverage and causal cross-market diagnostics; no order authority.

Uses frozen, previously collected broker episodes. This is NOT a strategy replay:
fixed horizons isolate predictive direction from stop/target engineering. All
variants and missing cases remain visible; no winner is selected or deployed.
"""
import argparse
import bisect
import collections
import datetime as dt
import hashlib
import html
import itertools
import json
import math
import statistics
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PLAN = ROOT / "docs/cost-context-plan-2026-11-05.json"
USD = {"EURUSD": -1, "GBPUSD": -1, "USDJPY": 1,
       "AUDUSD": -1, "USDCAD": 1, "NZDUSD": -1}


def ms(value):
    return int(dt.datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp() * 1000)


def quotes(source):
    """Same-millisecond end snapshot; never infer cross-side event ordering."""
    events = sorted([(x["ts_ms"], side, x["price_e5"] / 100000)
                     for side, name in enumerate(("bids", "asks"))
                     for x in source[name]], key=lambda x: x[0])
    sides = [None, None]
    result = []
    for at, group in itertools.groupby(events, key=lambda x: x[0]):
        for _, side, price in group:
            if not math.isfinite(price) or price <= 0:
                raise ValueError("invalid tick price")
            sides[side] = (at, price)
        if any(x is None for x in sides):
            continue
        if max(at - x[0] for x in sides) > 6000 or sides[0][1] > sides[1][1]:
            continue
        result.append((at, sides[0][1], sides[1][1]))
    return result


def first(qs, at, end):
    i = bisect.bisect_left(qs, (at,))
    return qs[i] if i < len(qs) and qs[i][0] < end else None


def prior(qs, at, max_age=60000):
    i = bisect.bisect_right(qs, (at, math.inf, math.inf)) - 1
    return qs[i] if i >= 0 and 0 <= at - qs[i][0] <= max_age else None


def mid(q):
    return (q[1] + q[2]) / 2


def sign(value):
    return 1 if value > 0 else -1 if value < 0 else 0


def peers(symbol, cls, classes):
    if cls in ("fx", "metal"):
        return {s: orientation for s, orientation in USD.items() if s != symbol}, 3
    if cls == "energy":
        return ({"SpotCrude": {"SpotBrent": 1},
                 "SpotBrent": {"SpotCrude": 1}}.get(symbol, {}), 1)
    return {s: 1 for s in classes[cls] if s != symbol}, 1


def factor(symbol, cls, start, decision, corpus, classes):
    required, minimum = peers(symbol, cls, classes)
    components = []
    for peer, orientation in required.items():
        qs = corpus.get((peer, start))
        if qs is None:
            continue  # Do not use a different market's later opening hour.
        opening = first(qs, start, start + 60000)
        last = prior(qs, decision)
        if opening is None or last is None or opening[0] >= last[0]:
            continue
        components.append(dict(symbol=peer, first_at=opening[0], last_at=last[0],
                               return_bps=(mid(last) / mid(opening) - 1) * 10000 * orientation))
    value = statistics.mean(x["return_bps"] for x in components) if len(components) >= minimum else None
    if value is not None:
        value *= USD[symbol] if cls == "fx" else -1 if cls == "metal" else 1
    return value, components


def outcome(entry, close, side, per_side):
    """Executable bid/ask includes spread once; costs normalised to entry mid."""
    a, b = (entry[2], close[1]) if side > 0 else (entry[1], close[2])
    gross = side * (b - a) / mid(entry) * 10000
    drag = (a + b) / mid(entry) * per_side
    return dict(mid_direction_correct=side * (mid(close) - mid(entry)) > 0,
                executable_before_fees_bps=gross, stress_after_slippage_bps=gross - drag,
                # Negative means even zero additional cost cannot rescue this outcome.
                break_even_per_side_bps=side * (b - a) / (a + b) * 10000,
                qualified_net_bps=None)


def evaluate(plan, corpus):
    rows = []
    for cls, symbols in plan["classes"].items():
        for symbol in symbols:
            for (s, start), qs in sorted(corpus.items()):
                if s != symbol:
                    continue
                mark = start + 3600000
                opening = first(qs, start, start + 60000)
                decision = first(qs, mark, mark + 5000)
                entry = first(qs, decision[0] + 1, mark + 5000) if decision else None
                own = mid(decision) - mid(opening) if decision and opening else None
                value, components = factor(symbol, cls, start, decision[0], corpus, plan["classes"]) if decision else (None, [])
                directions = dict(own_momentum=sign(own) if own is not None else None,
                                  peer_continuation=sign(value) if value is not None else None,
                                  peer_reversal=-sign(value) if value is not None else None,
                                  always_long=1)
                for horizon in plan["horizons_minutes"]:
                    close_mark = mark + horizon * 60000
                    close = first(qs, close_mark, close_mark + 5000)
                    for variant in plan["variants"]:
                        side = directions[variant]
                        reason = ("missing_decision_quote" if not decision else
                                  "missing_next_fresh_entry" if not entry else
                                  "missing_predictor" if side is None else
                                  "flat_predictor" if not side else
                                  "missing_close_quote" if not close else "observed")
                        row = dict(symbol=symbol, asset_class=cls, start_ms=start,
                                   horizon_minutes=horizon, variant=variant, side=side,
                                   status=reason, decision_ms=decision[0] if decision else None,
                                   entry_ms=entry[0] if entry else None,
                                   exit_ms=close[0] if close else None,
                                   peer_components=components if variant.startswith("peer") else [])
                        row["outcome"] = outcome(entry, close, side, plan["slippage_bps_per_side"]) if reason == "observed" else None
                        rows.append(row)
    return rows


def aggregate(rows):
    grouped = collections.defaultdict(list)
    for r in rows:
        grouped[(r["asset_class"], r["variant"], r["horizon_minutes"])].append(r)
    results = []
    for key, rs in sorted(grouped.items()):
        obs = [r for r in rs if r["outcome"] is not None]
        n = len(obs)
        symbols = sorted({r["symbol"] for r in rs})
        values = [r["outcome"]["stress_after_slippage_bps"] for r in obs]
        results.append(dict(asset_class=key[0], variant=key[1], horizon_minutes=key[2],
                            expected=len(rs), observed=n,
                            missing=dict(collections.Counter(r["status"] for r in rs if r["outcome"] is None)),
                            direction_hits=sum(r["outcome"]["mid_direction_correct"] for r in obs),
                            before_fees_winners=sum(r["outcome"]["executable_before_fees_bps"] > 0 for r in obs),
                            stress_winners=sum(x > 0 for x in values),
                            stress_mean_bps=statistics.mean(values) if n else None,
                            per_symbol={s: dict(observed=sum(r["symbol"] == s for r in obs),
                                                stress_winners=sum(r["symbol"] == s and r["outcome"]["stress_after_slippage_bps"] > 0 for r in obs)) for s in symbols},
                            financial_pass=False))
    return results


def matched_controls(rows):
    """Compare the same episodes, never a filtered peer arm to all controls."""
    indexed = {(r["symbol"], r["start_ms"], r["horizon_minutes"], r["variant"]): r for r in rows}
    groups = collections.defaultdict(list)
    for r in rows:
        if not r["variant"].startswith("peer") or r["outcome"] is None:
            continue
        for control in ("own_momentum", "always_long"):
            other = indexed[(r["symbol"], r["start_ms"], r["horizon_minutes"], control)]
            if other["outcome"] is not None:
                groups[(r["asset_class"], r["variant"], r["horizon_minutes"], control)].append((r, other))
    return [dict(asset_class=k[0], variant=k[1], horizon_minutes=k[2], control=k[3],
                 matched=len(pairs), changed_direction=sum(a["side"] != b["side"] for a, b in pairs),
                 mean_stress_delta_bps=statistics.mean(a["outcome"]["stress_after_slippage_bps"] - b["outcome"]["stress_after_slippage_bps"] for a, b in pairs),
                 peer_stress_wins=sum(a["outcome"]["stress_after_slippage_bps"] > 0 for a, _ in pairs),
                 control_stress_wins=sum(b["outcome"]["stress_after_slippage_bps"] > 0 for _, b in pairs))
            for k, pairs in sorted(groups.items())]


def receipt_coverage(paths):
    rows = {}
    for path in paths:
        for row in json.loads(path.read_text())["rows"]:
            key = (row["strategy"], row["account_id"], row["entry_key"])
            if key in rows and rows[key] != row:
                raise ValueError("conflicting receipt")
            rows[key] = row
    groups = collections.defaultdict(list)
    for row in rows.values():
        if row["kind"] == "open" and row.get("fill_price") is not None:
            groups[(row["strategy"], row["account_id"], row["symbol"])].append(row)
    result = []
    for key, rs in sorted(groups.items(), key=lambda x: str(x[0])):
        measured = []
        for r in rs:
            a = r.get("entry_cost")
            if a is None:
                continue
            if not all(math.isfinite(a[k]) for k in ("dispatch_price", "fill_price", "dispatch_to_fill_bps")):
                raise ValueError("nonfinite entry cost")
            if a["dispatch_price"] <= 0 or a["fill_price"] <= 0 or r["side"] not in ("buy", "sell") or not 0 <= a["dispatch_quote_age_ms"] <= 6000:
                raise ValueError("invalid dispatch evidence")
            expected = (1 if r["side"] == "buy" else -1) * (a["fill_price"] - a["dispatch_price"]) / a["dispatch_price"] * 10000
            if not math.isclose(expected, a["dispatch_to_fill_bps"], abs_tol=1e-8):
                raise ValueError("dispatch slippage mismatch")
            if a["decision_to_fill_bps"] is not None and not math.isclose(a["decision_to_dispatch_bps"] + a["dispatch_to_fill_bps"], a["decision_to_fill_bps"], abs_tol=1e-8):
                raise ValueError("cost decomposition mismatch")
            measured.append(a)
        slips = [a["dispatch_to_fill_bps"] for a in measured]
        result.append(dict(strategy=key[0], account=key[1], symbol=key[2], filled_opens=len(rs), measured_entries=len(measured),
                           dispatch_slippage_bps_median=statistics.median(slips) if slips else None,
                           dispatch_slippage_bps_max=max(slips) if slips else None,
                           round_trip_calibrated=False))
    return dict(scope="Eight bounded newest-500 order views; not an unbiased or complete execution sample.",
                unique_rows=len(rows), filled_opens=sum(x["filled_opens"] for x in result),
                measured_entries=sum(x["measured_entries"] for x in result), groups=result,
                legacy_market_closes=sum(r["kind"] == "close" and r.get("fill_price") is not None for r in rows.values()),
                broker_exits=sum(r["kind"] == "broker_exit" for r in rows.values()),
                calibrated_per_side_bps=None,
                blockers=["No certified exit-dispatch distribution or complete paired fee lifecycle in these views.",
                          "Legacy close slippage and broker SL/TP-level gaps are different benchmarks; do not pool them with entry dispatch slippage.",
                          "Selected demo entries cannot calibrate live, other symbols, or unobserved sizes/regimes."])


def report(result, path):
    def table(rows, keys):
        return '<div class="scroll"><table><tr>' + ''.join('<th>' + html.escape(k) + '</th>' for k in keys) + '</tr>' + ''.join('<tr>' + ''.join('<td>' + html.escape(str(r.get(k))) + '</td>' for k in keys) + '</tr>' for r in rows) + '</table></div>'
    c = result["cost_coverage"]
    three_hour = [r for r in result["summary"] if r["variant"] == "peer_continuation" and r["horizon_minutes"] == 180]
    findings = []
    for r in three_hour:
        mean = f'{r["stress_mean_bps"]:.2f}bp' if r["stress_mean_bps"] is not None else 'unknown'
        findings.append(f'{r["asset_class"]}: {r["stress_winners"]}/{r["observed"]} observed stress winners, mean {mean}')
    symbols = []
    for r in result["summary"]:
        for s, counts in r["per_symbol"].items():
            symbols.append(dict(symbol=s, variant=r["variant"], horizon_minutes=r["horizon_minutes"], **counts))
    path.write_text('''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Jiaoyi cost and context evidence</title><style>body{font:16px/1.5 system-ui;max-width:1300px;margin:30px auto;padding:18px;color:#203040}table{border-collapse:collapse;font-size:13px}td,th{padding:8px;border-bottom:1px solid #ddd;text-align:left}.scroll{overflow:auto}p{max-width:1000px}code{overflow-wrap:anywhere}</style><h1>Jiaoyi: costs and class-specific context</h1>'''
                    + '<p>' + html.escape(result["trace"]) + '</p><h2>TLDR</h2>'
                    + f'<p>0 of 5 asset classes have qualified evidence of the 50% net-win target. {c["measured_entries"]} of {c["filled_opens"]} filled entry records have dispatch attribution. No round-trip cost estimate is certified; the 24bp/side production allowance is unchanged.</p>'
                    + '<p>This run tests related-market information at 30-, 60- and 180-minute horizons on the same two previously inspected development dates. It separates midpoint direction, executable bid/ask outcomes before fees, and the unchanged 24bp/side stress scenario. These are fixed-horizon diagnostics, not trades with the programme’s SL/TP, sizing, margin or trailing rules. They do not establish goal progress or justify promotion.</p>'
                    + '<p>Three-hour peer continuation: ' + html.escape('; '.join(findings)) + '. NatGas has no supported peer predictor. These tiny, previously seen samples do not qualify a class. Matched-control results below distinguish added decision value from repeated own-momentum decisions.</p>'
                    + '<h2>Three-hour peer-continuation snapshot</h2>'
                    + table(three_hour, ["asset_class", "expected", "observed", "direction_hits", "before_fees_winners", "stress_winners", "stress_mean_bps", "missing"])
                    + '<h2>Does the peer add information?</h2><p>Comparisons below use only identical episodes observed in both arms. A positive difference is a descriptive development result, not proof of an incremental edge. Zero changed directions means no added decision value on those episodes.</p><details><summary>All matched-control comparisons</summary>'
                    + table(result["matched_controls"], ["asset_class", "variant", "horizon_minutes", "control", "matched", "changed_direction", "mean_stress_delta_bps", "peer_stress_wins", "control_stress_wins"]) + '</details>'
                    + '<h2>Class results — every registered variant and horizon</h2><p>Always-long and own-momentum are controls. Peer continuation and reversal are competing hypotheses. No retrospective winner is selected. Counts share dates and symbols; they are correlated, small samples.</p>'
                    + table(result["summary"], ["asset_class", "variant", "horizon_minutes", "expected", "observed", "direction_hits", "before_fees_winners", "stress_winners", "stress_mean_bps", "missing"])
                    + '<details><summary>Every symbol, variant and horizon</summary>'
                    + table(symbols, ["symbol", "variant", "horizon_minutes", "observed", "stress_winners"]) + '</details>'
                    + '<h2>Actual receipt coverage</h2><p>Missing samples remain unknown, not zero. Current demo attribution cannot be transferred to live or to historical 2025 fills. Exit benchmarks and commissions remain unresolved.</p>'
                    + table(c["groups"], ["strategy", "account", "symbol", "filled_opens", "measured_entries", "dispatch_slippage_bps_median", "dispatch_slippage_bps_max"])
                    + '<h2>Data and decision limits</h2><p>FX: an equal-weight USD peer factor, excluding the target. Metals: inverse USD proxy, missing real yields. Oil: the other crude benchmark; NatGas deliberately has no oil proxy. Crypto: the other cryptocurrency, without attention/funding data. Indices: peers with the same opening window only. Common exposure or co-movement is not proof of a predictive lead.</p>'
                    + '<p>Only observations available by the decision may enter a factor. The entry is a strictly later fresh quote. Missing predictors, fills and exits are retained. Broker-side history is not measured local dispatch latency. Commission/funding and historical contract qualification remain unknown. All per-symbol outcomes, peer timestamps, source/code hashes and exclusions are in cost-context-results.json.</p>'
                    + '<h2>Failure prevention and next gates</h2><ol><li>Obtain representative, paired entry/exit/fee evidence separately by account environment, symbol and size; do not lower fees to hit the target.</li><li>Obtain historical as-published explanatory inputs and expectations for the selected class; do not backfill revised values or pretend co-movement is predictive.</li><li>Freeze one economically supported candidate and verify it against controls, including costs and per-symbol results.</li><li>Only after development passes, open separate-time evaluation and then demo shadow checks. Preserve losing results and missing observations.</li><li>Promote only with evidence and applicable owner authority. The original 73% / 1:1.38 per strategy/account goal remains unchanged.</li></ol>'
                    + '<p>Sources and mechanism-specific findings: <a href="https://github.com/ang-kl/jiaoyi/blob/main/docs/cost-context-findings-2026-11-05.md">research findings and five gates</a>. No new broker history, database retention, model API call or live strategy change. Draft for review.</p></html>')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--sources", type=Path, required=True)
    p.add_argument("--receipts", type=Path, required=True)
    p.add_argument("--binary", type=Path, required=True, help="native class_prediction_study verifier")
    p.add_argument("--output", type=Path, required=True)
    a = p.parse_args()
    plan_bytes = PLAN.read_bytes()
    plan = json.loads(plan_bytes)
    corpus, provenance = {}, []
    for name, digest in plan["input_files"].items():
        path = a.sources / name
        if path.stat().st_size > 32000000 or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
            raise ValueError("source file differs from frozen plan")
        job = json.loads(path.read_text())
        # Existing native validator checks tick bounds/grid, source digest and
        # exact original study. No permissive Python reimplementation of that gate.
        verified = json.loads(subprocess.run([str(a.binary), str(path)], check=True, capture_output=True, text=True, timeout=60).stdout)
        req = job["request"]
        key = (req["symbol"], ms(req["from"]))
        if key in corpus:
            raise ValueError("duplicate source episode")
        corpus[key] = quotes(job["result"]["input"])
        provenance.append(dict(file=name, file_sha256=digest, input_sha256=verified["input_sha256"], source_algorithm_sha256=verified["source_algorithm_sha256"], verifier_algorithm_sha256=verified["replay_algorithm_sha256"]))
    paths = sorted(a.receipts.glob("orders-*.json"))
    if len(paths) != 8:
        raise ValueError("eight bounded account/strategy receipt views required")
    rows = evaluate(plan, corpus)
    result = dict(trace="№18,260 §§1–6 · " + dt.datetime.now(dt.timezone(dt.timedelta(hours=8))).strftime("%d-%m-%Y %H:%M SGT") + " · by Codex (GPT-6), effort not exposed",
                  plan_sha256=hashlib.sha256(plan_bytes).hexdigest(),
                  analysis_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  receipt_files={p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
                  provenance=provenance, cost_coverage=receipt_coverage(paths),
                  summary=aggregate(rows), matched_controls=matched_controls(rows), rows=rows, financial_pass=False)
    a.output.mkdir(parents=True, exist_ok=True)
    (a.output / "cost-context-results.json").write_text(json.dumps(result, indent=2, allow_nan=False))
    report(result, a.output / "Jiaoyi_Cost_Context_Report.html")
    print(json.dumps(dict(sources=len(corpus), diagnostic_rows=len(rows), measured_entries=result["cost_coverage"]["measured_entries"], financial_pass=False)))


if __name__ == "__main__":
    main()
