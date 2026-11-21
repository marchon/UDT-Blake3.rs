#!/usr/bin/env bash
# Short WAN probe via wan_probe example (Blake3 feature).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOSTPORT="${1:?usage: wan-probe.sh host:port [bytes]}"
BYTES="${2:-8388608}"
PROBE="${PROBE:-$ROOT/target/release/examples/wan_probe}"
if [[ ! -x "$PROBE" ]]; then
  echo "build: cargo build -p udt-socket --example wan_probe --release --features blake3-reliability" >&2
  exit 1
fi
echo "wan-probe → $HOSTPORT bytes=$BYTES"
exec "$PROBE" "$HOSTPORT" "$BYTES"
