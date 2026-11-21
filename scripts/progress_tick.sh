#!/usr/bin/env bash
# Append one progress event to progress/progress.jsonl
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/progress/progress.jsonl"
mkdir -p "$ROOT/progress"

TEAM="${1:?team}"
AGENT="${2:?agent}"
TASK="${3:?task}"
MILESTONE="${4:?milestone}"
PCT="${5:?pct}"
STATUS="${6:?status}"
BLOCKERS="${7:-none}"
NOTE="${8:-}"

TS="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
# Escape note for JSON (minimal)
NOTE_ESC="${NOTE//\\/\\\\}"
NOTE_ESC="${NOTE_ESC//\"/\\\"}"

printf '{"ts":"%s","team":"%s","agent":"%s","task":"%s","milestone":"%s","pct":%s,"status":"%s","blockers":"%s","note":"%s"}\n' \
  "$TS" "$TEAM" "$AGENT" "$TASK" "$MILESTONE" "$PCT" "$STATUS" "$BLOCKERS" "$NOTE_ESC" >>"$OUT"
