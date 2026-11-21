#!/usr/bin/env bash
# Run one PureRust ↔ CppUdt4 interop cell and write evidence under artifacts/interop/.
# usage: run_interop_cell.sh rust-cpp|cpp-rust|rust-rust [bytes]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
CELL="${1:-rust-cpp}"
BYTES="${2:-65536}"
CPP="$ROOT/crates/udt-interop/cpp"
EVID="$ROOT/artifacts/interop/$CELL"
mkdir -p "$EVID"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

dd if=/dev/urandom of="$TMP/in.bin" bs=1 count="$BYTES" status=none 2>/dev/null \
  || dd if=/dev/urandom of="$TMP/in.bin" bs=1024 count=$(( (BYTES + 1023) / 1024 )) status=none
# trim to exact size
python3 - "$TMP/in.bin" "$BYTES" <<'PY'
import sys
path, n = sys.argv[1], int(sys.argv[2])
with open(path, 'rb') as f:
    data = f.read(n)
with open(path, 'wb') as f:
    f.write(data)
PY

need_cpp=0
case "$CELL" in
  rust-cpp|cpp-rust) need_cpp=1 ;;
esac
if [[ "$need_cpp" == 1 ]]; then
  if [[ ! -x "$CPP/stream_server" || ! -x "$CPP/stream_client" ]]; then
    "$ROOT/crates/udt-interop/scripts/build_vendor.sh"
  fi
fi

PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"
STATUS=fail
NOTE=""

run_rust_listen() {
  cargo run -q -p udt-cli -- -l "127.0.0.1:$PORT" -o "$TMP/out.bin" >"$EVID/server.log" 2>&1 &
  echo $! >"$TMP/pid"
  sleep 0.5
}
run_rust_send() {
  cargo run -q -p udt-cli -- "127.0.0.1:$PORT" -i "$TMP/in.bin" >"$EVID/client.log" 2>&1
}
run_cpp_listen() {
  "$CPP/stream_server" "$PORT" "$TMP/out.bin" "$BYTES" >"$EVID/server.log" 2>&1 &
  echo $! >"$TMP/pid"
  sleep 0.4
}
run_cpp_send() {
  "$CPP/stream_client" 127.0.0.1 "$PORT" "$TMP/in.bin" >"$EVID/client.log" 2>&1
}

case "$CELL" in
  rust-cpp)
    run_cpp_listen
    run_rust_send || true
    # C++ server exits after expect_bytes
    wait "$(cat "$TMP/pid")" 2>/dev/null || true
    ;;
  cpp-rust)
    run_rust_listen
    run_cpp_send || true
    for _ in $(seq 1 40); do
      if cmp -s "$TMP/in.bin" "$TMP/out.bin" 2>/dev/null; then break; fi
      sleep 0.5
    done
    kill "$(cat "$TMP/pid")" 2>/dev/null || true
    wait "$(cat "$TMP/pid")" 2>/dev/null || true
    ;;
  rust-rust)
    run_rust_listen
    run_rust_send || true
    for _ in $(seq 1 40); do
      if cmp -s "$TMP/in.bin" "$TMP/out.bin" 2>/dev/null; then break; fi
      sleep 0.5
    done
    kill "$(cat "$TMP/pid")" 2>/dev/null || true
    wait "$(cat "$TMP/pid")" 2>/dev/null || true
    ;;
  *)
    echo "unknown cell: $CELL" >&2
    exit 2
    ;;
esac
sleep 0.2

if cmp -s "$TMP/in.bin" "$TMP/out.bin"; then
  STATUS=pass
  NOTE="byte-exact $BYTES"
else
  IN_N=$(wc -c <"$TMP/in.bin" | tr -d ' ')
  OUT_N=0
  [[ -f "$TMP/out.bin" ]] && OUT_N=$(wc -c <"$TMP/out.bin" | tr -d ' ')
  NOTE="mismatch in=$IN_N out=$OUT_N"
fi

{
  echo "cell=$CELL"
  echo "status=$STATUS"
  echo "bytes=$BYTES"
  echo "port=$PORT"
  echo "note=$NOTE"
  echo "ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} | tee "$EVID/summary.txt"

cp "$TMP/in.bin" "$EVID/in.bin" 2>/dev/null || true
[[ -f "$TMP/out.bin" ]] && cp "$TMP/out.bin" "$EVID/out.bin" || true

[[ "$STATUS" == pass ]]
