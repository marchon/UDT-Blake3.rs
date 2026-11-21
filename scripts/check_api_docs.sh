#!/usr/bin/env bash
# Fail if API one-pagers still contain template placeholders, or if the
# docs/api/README.md index links to a missing page.
# Excludes docs/api/TEMPLATE.md (intentional placeholders).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
API="$ROOT/docs/api"
fail=0

if [[ ! -d "$API" ]]; then
  echo "error: missing $API" >&2
  exit 1
fi

# 1) No leftover TODO_FUNCTION_NAME outside the template.
while IFS= read -r -d '' f; do
  base="$(basename "$f")"
  [[ "$base" == "TEMPLATE.md" ]] && continue
  if grep -n -F 'TODO_FUNCTION_NAME' "$f" >/dev/null 2>&1; then
    echo "FAIL: $f contains TODO_FUNCTION_NAME"
    grep -n -F 'TODO_FUNCTION_NAME' "$f" || true
    fail=1
  fi
done < <(find "$API" -type f -name '*.md' -print0)

# 2) No unfilled TODO inside rust signature fences (```rust ... ```).
while IFS= read -r -d '' f; do
  base="$(basename "$f")"
  [[ "$base" == "TEMPLATE.md" ]] && continue
  awk -v file="$f" '
    BEGIN { in_rust=0; bad=0 }
    /^```rust/ { in_rust=1; next }
    /^```/ { in_rust=0; next }
    in_rust && /TODO/ {
      printf "FAIL: %s:%d: TODO in signature fence: %s\n", file, NR, $0
      bad=1
    }
    END { exit bad }
  ' "$f" || fail=1
done < <(find "$API" -type f -name '*.md' -print0)

# 3) Index links in docs/api/README.md must resolve.
INDEX="$API/README.md"
if [[ -f "$INDEX" ]]; then
  while IFS= read -r rel; do
    [[ -z "$rel" ]] && continue
    target="$API/$rel"
    if [[ ! -f "$target" ]]; then
      echo "FAIL: index links to missing page: $rel"
      fail=1
    fi
  done < <(grep -oE '\[[^]]+\]\(([^)]+\.md)\)' "$INDEX" | sed -E 's/.*\(([^)]+)\)/\1/' | grep -v 'TEMPLATE.md' || true)
fi

# 4) Required section headings on every one-pager (not README/TEMPLATE).
while IFS= read -r -d '' f; do
  base="$(basename "$f")"
  [[ "$base" == "TEMPLATE.md" || "$base" == "README.md" ]] && continue
  for heading in "## 1. Signature" "## 2. Semantics" "## 9. See also"; do
    if ! grep -qF "$heading" "$f"; then
      echo "FAIL: $f missing heading: $heading"
      fail=1
    fi
  done
done < <(find "$API" -type f -name '*.md' -print0)

if [[ "$fail" -ne 0 ]]; then
  echo "check_api_docs: failed" >&2
  exit 1
fi

echo "check_api_docs: ok"
