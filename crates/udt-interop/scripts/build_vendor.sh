#!/usr/bin/env bash
# Build vendored C++ UDT4 + thin stream peers for Phase 6 interop.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
VENDOR="$ROOT/crates/udt-interop/vendor/udt/udt4"
CPP="$ROOT/crates/udt-interop/cpp"

if [[ ! -d "$VENDOR/src" ]]; then
  echo "vendor missing; cloning eminence/udt @ 50638887…" >&2
  mkdir -p "$(dirname "$VENDOR")"
  git clone https://github.com/eminence/udt.git "$(dirname "$VENDOR")/udt"
  git -C "$(dirname "$VENDOR")/udt" checkout 50638887518f52dd6f6099050cfff75f9f7db08d
fi

UNAME_S="$(uname -s)"
UNAME_M="$(uname -m)"
if [[ "$UNAME_S" == "Darwin" ]]; then
  OS=OSX
  ARCH=ARM_BASE
elif [[ "$UNAME_M" == "x86_64" ]]; then
  OS=LINUX
  ARCH=AMD64
else
  OS=LINUX
  ARCH=ARM_BASE
fi

echo "building libudt (os=$OS arch=$ARCH)…"
make -C "$VENDOR/src" os="$OS" arch="$ARCH" -j"$(sysctl -n hw.ncpu 2>/dev/null || nproc)"

echo "building stream peers…"
make -C "$CPP" UDT_ROOT="$VENDOR" OS="$OS" ARCH="$ARCH"

echo "ok: $CPP/stream_server $CPP/stream_client"
ls -la "$CPP/stream_server" "$CPP/stream_client"
