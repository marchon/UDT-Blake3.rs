#!/usr/bin/env bash
# Localhost udtcat round-trip smoke (Blake3 default).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
UDTCAT="${UDTCAT:-$ROOT/target/release/udtcat}"
if [[ ! -x "$UDTCAT" ]]; then
  UDTCAT="$ROOT/dist/darwin-aarch64/udtcat"
fi
if [[ ! -x "$UDTCAT" ]]; then
  echo "build first: cargo build -p udt-cli --release" >&2
  exit 1
fi
PORT="${PORT:-19123}"
TMP="$(mktemp -d)"
trap 'kill "$SRV_PID" 2>/dev/null || true; rm -rf "$TMP"' EXIT
dd if=/dev/urandom of="$TMP/in.bin" bs=1024 count=256 status=none
"$UDTCAT" -l "127.0.0.1:$PORT" -o "$TMP/out.bin" &
SRV_PID=$!
sleep 0.3
"$UDTCAT" "127.0.0.1:$PORT" -i "$TMP/in.bin"
sleep 0.5
kill "$SRV_PID" 2>/dev/null || true
wait "$SRV_PID" 2>/dev/null || true
if cmp -s "$TMP/in.bin" "$TMP/out.bin"; then
  echo "OK localhost smoke: $(wc -c < "$TMP/in.bin") bytes match"
else
  echo "FAIL size/content mismatch" >&2
  ls -l "$TMP" >&2
  exit 1
fi
