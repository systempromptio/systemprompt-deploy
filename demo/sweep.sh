#!/bin/bash
# Sweep harness — runs every demo script under demo/**/*.sh (excluding
# _common.sh, 00-preflight.sh, 01-seed-data.sh, and sweep.sh itself).
# Captures stdout+stderr per demo to /tmp/demo-sweep/<category>-<name>.log
# and prints a green/red summary at the end.
#
# Usage: ./demo/sweep.sh

set -u

DEMO_ROOT="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="/tmp/demo-sweep"
mkdir -p "$OUT_DIR"
rm -f "$OUT_DIR"/*.log

PASS=0
FAIL=0
SCRIPTS=()

while IFS= read -r -d '' script; do
  name=$(basename "$script" .sh)
  parent=$(basename "$(dirname "$script")")
  case "$name" in
    _common|sweep|00-preflight|01-seed-data) continue ;;
  esac
  if [[ "$parent" == "demo" ]]; then
    continue
  fi
  SCRIPTS+=("$parent/$name")
done < <(find "$DEMO_ROOT" -mindepth 2 -maxdepth 2 -name '*.sh' -print0 | sort -z)

echo "Running ${#SCRIPTS[@]} demos…"

for entry in "${SCRIPTS[@]}"; do
  parent="${entry%/*}"
  name="${entry#*/}"
  script="$DEMO_ROOT/$parent/$name.sh"
  log="$OUT_DIR/${parent}-${name}.log"
  if bash "$script" > "$log" 2>&1; then
    PASS=$((PASS + 1))
    printf '  \033[0;32m✓\033[0m %-45s → %s\n' "$entry" "$log"
  else
    FAIL=$((FAIL + 1))
    printf '  \033[0;31m✗\033[0m %-45s → %s\n' "$entry" "$log"
  fi
done

echo ""
echo "Pass: $PASS    Fail: $FAIL    Logs: $OUT_DIR/"
exit $((FAIL > 0 ? 1 : 0))
