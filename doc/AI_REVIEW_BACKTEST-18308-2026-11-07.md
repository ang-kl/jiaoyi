# Independent AI review instruction

№18,308 §8 · 07-11-2026 08:21 SGT. By Codex (GPT-6), effort not exposed.

Review [the backtest report](BACKTEST_REPORT-18308-2026-11-07.md) and
[manifest](BACKTEST_EVIDENCE-18308-2026-11-07.json), then follow their source links.
Baseline: main after PR #175, commit e8fdf1b448876aa189d2853b2cf44f2a398c803b.
Read later commits and both AI logs before allocating a new serial.

Do not accept the author's conclusions by default. Separate source-backed
facts, mathematical deductions, modelling assumptions and missing evidence.
The user requests review; this prompt does not independently authorise trading,
collection, deployment, changes to risk values or weakening the research seal.

Answer these questions with file/function references and minimal counterexamples:

1. Do the two market replays actually contain trades? Verify C/R and session
   vetoes. Is any win-rate or improvement claim supported by those outputs?
2. Do the five-quote oil fixtures prove anything about a 13-hour real market
   path? Check censoring, ambiguity flags and qualified versus scenario returns.
3. Does the new cost recheck preserve production's frozen payload and refusal
   semantics? Assess use of the next quote as a dispatch proxy, including
   deadline, freshness, account revision, connection and restart limitations.
4. Are declared preparation timestamps, opening volume and cost values adequate
   provenance? Challenge their authenticity, causal timing and the 45-day view.
5. Are the exit and account-admission gaps material? Which have an actionable
   deterministic regression now, and which need unavailable historical evidence?
6. Are the earlier oil candidate's results fairly reported without conflating
   its rules with production S3 or turning cost sensitivities into a new edge?
7. What is the smallest justified next deliverable? Specify required evidence,
   a small number of meaningful tests and predeclared stop conditions. Avoid
   framework replacement, parameter search or fresh batches without a reason.

Return: TLDR; claim-by-claim verdict (Supported / Contradicted / Not Verifiable);
severity-ranked concrete defects with reproductions; data gaps; smallest next
step; whether any financial promotion is justified. Do not infer market wins
from synthetic tests, CI, zero-entry episodes or missing raw data. Preserve the
50% class/symbol research target and original73% /1:1.38 latest20 account goal.
