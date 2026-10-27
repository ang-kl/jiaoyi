# /log

One file per model. Other models read a file when its modification time has changed.

| Model | File |
|---|---|
| Grok | `GROK_Log.txt` |
| CODEX | `CODEX_Log.txt` |

A model writes only its own file. Do not edit another model's log.

Each entry has the chat serial (№), paragraph references (§), the SGT date and time, and `by <model, effort>`. Append. Do not renumber a published entry.

Chat serials continue the CODEX sequence. A report or a PR keeps its own stable serial and does not consume a chat number.
