# Progress event schema

Append-only file: `progress/progress.jsonl`  
One JSON object per line.

## Fields

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `ts` | string (RFC3339 UTC) | yes | Event timestamp |
| `team` | `"A"` \| `"B"` \| `"C"` \| `"ORCH"` | yes | Team id |
| `agent` | string | yes | Agent id (e.g. `A1-codec`) |
| `task` | string | yes | Slug of current task |
| `milestone` | string | yes | Phase id (`phase-0` … `phase-7`) |
| `pct` | number 0–100 | yes | Task completion percent |
| `status` | `"running"` \| `"blocked"` \| `"done"` \| `"idle"` | yes | State |
| `blockers` | string | yes | `"none"` or short reason |
| `note` | string | no | Free-text update |

## Example

```json
{"ts":"2026-11-21T19:14:00Z","team":"A","agent":"A1-codec","task":"handshake-header-codec","milestone":"phase-1","pct":40,"status":"running","blockers":"none","note":"ACK type round-trip green"}
```

## Console line format

```text
[HH:MM:SSZ] team=A agent=A1-codec task=handshake-header-codec milestone=phase-1 pct=40 status=running blockers=none
```
