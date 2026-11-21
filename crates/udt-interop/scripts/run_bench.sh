#!/usr/bin/env bash
# Lab bench: PureRust localhost throughput for a fixed payload (not a 1 Gbps claim).
# Writes artifacts/bench/summary.txt with MiB/s.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
EVID="$ROOT/artifacts/bench"
mkdir -p "$EVID"
BYTES="${BENCH_BYTES:-8388608}" # 8 MiB
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

python3 - "$TMP/in.bin" "$BYTES" <<'PY'
import os, sys
path, n = sys.argv[1], int(sys.argv[2])
with open(path, 'wb') as f:
    f.write(os.urandom(n))
PY

PORT="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')"
cargo run -q -p udt-cli -- -l "127.0.0.1:$PORT" -o "$TMP/out.bin" >"$EVID/server.log" 2>&1 &
PID=$!
sleep 0.4

START_NS="$(python3 -c 'import time; print(time.time_ns())')"
cargo run -q -p udt-cli -- "127.0.0.1:$PORT" -i "$TMP/in.bin" >"$EVID/client.log" 2>&1
# wait for receiver idle complete
for _ in $(seq 1 60); do
  if cmp -s "$TMP/in.bin" "$TMP/out.bin" 2>/dev/null; then break; fi
  sleep 0.25
done
END_NS="$(python3 -c 'import time; print(time.time_ns())')"
kill "$PID" 2>/dev/null || true
wait "$PID" 2>/dev/null || true

python3 - "$BYTES" "$START_NS" "$END_NS" "$EVID/summary.txt" "$TMP/in.bin" "$TMP/out.bin" <<'PY'
import sys, pathlib
nbytes, t0, t1, outp, inn, outn = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3]), sys.argv[4], sys.argv[5], sys.argv[6]
ok = pathlib.Path(inn).read_bytes() == pathlib.Path(outn).read_bytes() if pathlib.Path(outn).exists() else False
sec = max((t1 - t0) / 1e9, 1e-9)
mib = nbytes / (1024 * 1024)
mbps = (nbytes * 8 / sec) / 1e6
mib_s = mib / sec
lines = [
    f"status={'pass' if ok else 'fail'}",
    f"bytes={nbytes}",
    f"elapsed_sec={sec:.6f}",
    f"throughput_MiB_s={mib_s:.3f}",
    f"throughput_Mbps={mbps:.3f}",
    f"path=localhost_pure_rust",
    f"note=lab localhost; not a 1Gbps WAN claim",
]
pathlib.Path(outp).write_text("\n".join(lines) + "\n")
print("\n".join(lines))
sys.exit(0 if ok else 1)
PY
