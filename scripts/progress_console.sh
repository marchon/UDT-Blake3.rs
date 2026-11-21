#!/usr/bin/env bash
# Minute-level console: show which teams/agents work on what.
# Writes human board to progress/console.log and mirrors to stdout.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
JSONL="$ROOT/progress/progress.jsonl"
LOG="$ROOT/progress/console.log"
STATE="$ROOT/progress/latest_by_agent.json"
mkdir -p "$ROOT/progress"
touch "$JSONL" "$LOG"

INTERVAL="${PROGRESS_INTERVAL_SEC:-60}"
ONCE="${1:-}"

render() {
  local now
  now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  python3 - "$JSONL" "$STATE" "$now" <<'PY'
import json, sys, collections
from pathlib import Path
jsonl, state_path, now = sys.argv[1:4]
latest = {}
path = Path(jsonl)
if path.exists():
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            ev = json.loads(line)
        except json.JSONDecodeError:
            continue
        latest[ev.get("agent", "?")] = ev
Path(state_path).write_text(json.dumps(latest, indent=2, sort_keys=True) + "\n")

teams = collections.OrderedDict(A=[], B=[], C=[], ORCH=[])
for agent, ev in sorted(latest.items()):
    teams.setdefault(ev.get("team", "?"), []).append(ev)

print("=" * 78)
print(f"UDT pure-Rust progress board  @ {now}  (agents={len(latest)})")
print("=" * 78)
for team, rows in teams.items():
    if not rows and team == "?":
        continue
    print(f"\n## Team {team}")
    if not rows:
        print("  (no agents reporting)")
        continue
    for ev in sorted(rows, key=lambda e: e.get("agent", "")):
        ts = ev.get("ts", "")
        hh = ts[11:19] + "Z" if len(ts) >= 19 else ts
        print(
            f"  [{hh}] agent={ev.get('agent')} task={ev.get('task')} "
            f"milestone={ev.get('milestone')} pct={ev.get('pct')} "
            f"status={ev.get('status')} blockers={ev.get('blockers')}"
        )
        note = ev.get("note") or ""
        if note:
            print(f"           note: {note}")
print()
PY
}

if [[ "$ONCE" == "--once" ]]; then
  board="$(render)"
  printf "%s\n" "$board" | tee -a "$LOG"
  exit 0
fi

echo "progress_console: interval=${INTERVAL}s  log=$LOG  jsonl=$JSONL"
while true; do
  board="$(render)"
  printf "%s\n" "$board" | tee -a "$LOG"
  sleep "$INTERVAL"
done
