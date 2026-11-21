#!/usr/bin/env bash
# Timed full-file send with udtcat (Blake3 default).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOSTPORT="${1:?usage: wan-iso.sh host:port /path/to/file}"
FILE="${2:?usage: wan-iso.sh host:port /path/to/file}"
UDTCAT="${UDTCAT:-$ROOT/target/release/udtcat}"
if [[ ! -x "$UDTCAT" ]]; then
  echo "build first: cargo build -p udt-cli --release" >&2
  exit 1
fi
BYTES="$(wc -c < "$FILE" | tr -d ' ')"
echo "udtcat ISO-style send → $HOSTPORT file=$FILE bytes=$BYTES"
/usr/bin/time -p "$UDTCAT" "$HOSTPORT" -i "$FILE"
