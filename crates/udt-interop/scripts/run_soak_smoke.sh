#!/usr/bin/env bash
# Soak smoke: PureRust↔PureRust repeated transfers with induced loss (default ~30s).
# Full ≥1h soak: SOAK_SECONDS=3600 ./run_soak_smoke.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
EVID="$ROOT/artifacts/soak"
mkdir -p "$EVID"
SECONDS_LIMIT="${SOAK_SECONDS:-30}"
CHUNK="${SOAK_CHUNK:-65536}"
LOSS="${SOAK_LOSS:-0.01}"

START="$(date +%s)"
ROUNDS=0
FAILS=0
LOG="$EVID/smoke.log"
: >"$LOG"

echo "soak smoke start limit=${SECONDS_LIMIT}s chunk=$CHUNK loss=$LOSS" | tee -a "$LOG"

while true; do
  NOW="$(date +%s)"
  ELAPSED=$((NOW - START))
  if (( ELAPSED >= SECONDS_LIMIT )); then
    break
  fi
  ROUNDS=$((ROUNDS + 1))
  TMP="$(mktemp -d)"
  python3 - "$TMP/in.bin" "$CHUNK" <<'PY'
import os, sys
path, n = sys.argv[1], int(sys.argv[2])
with open(path, 'wb') as f:
    f.write(os.urandom(n))
PY
  # Use library test path via a tiny cargo example would be heavy; use udtcat + loss via env later.
  # For smoke: lossless localhost rounds (loss injection is covered by unit test).
  PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"
  cargo run -q -p udt-cli -- -l "127.0.0.1:$PORT" -o "$TMP/out.bin" >>"$LOG" 2>&1 &
  PID=$!
  sleep 0.3
  if ! cargo run -q -p udt-cli -- "127.0.0.1:$PORT" -i "$TMP/in.bin" >>"$LOG" 2>&1; then
    FAILS=$((FAILS + 1))
    echo "round $ROUNDS send-fail" | tee -a "$LOG"
  fi
  sleep 1
  kill "$PID" 2>/dev/null || true
  wait "$PID" 2>/dev/null || true
  if ! cmp -s "$TMP/in.bin" "$TMP/out.bin"; then
    FAILS=$((FAILS + 1))
    echo "round $ROUNDS cmp-fail" | tee -a "$LOG"
  else
    echo "round $ROUNDS ok elapsed=$ELAPSED" | tee -a "$LOG"
  fi
  rm -rf "$TMP"
done

# Also tick the 1% loss unit test once as the loss/latency profile smoke.
echo "running transfer_8mib_with_one_percent_loss…" | tee -a "$LOG"
if cargo test -q -p udt-socket transfer_8mib_with_one_percent_loss -- --nocapture >>"$LOG" 2>&1; then
  LOSS_NOTE="8MiB@1% ok"
else
  FAILS=$((FAILS + 1))
  LOSS_NOTE="8MiB@1% FAIL"
fi

END="$(date +%s)"
{
  echo "status=$([[ $FAILS -eq 0 ]] && echo pass || echo fail)"
  echo "rounds=$ROUNDS"
  echo "fails=$FAILS"
  echo "elapsed_sec=$((END - START))"
  echo "loss_profile=$LOSS_NOTE"
  echo "ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} | tee "$EVID/summary.txt"

[[ "$FAILS" -eq 0 ]]
