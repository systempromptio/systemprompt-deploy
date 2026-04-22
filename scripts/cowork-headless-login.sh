#!/bin/bash
# Headlessly mint a Cowork PAT using the CLI session JWT and wire it into sp-cowork-auth.
#
# Flow:
#   1. Get a fresh admin JWT (either from .systemprompt/sessions/index.json if still valid,
#      or by calling `systemprompt admin session login --token-only`).
#   2. POST /admin/devices/pats with Authorization: Bearer <JWT> to mint a PAT.
#   3. Run `sp-cowork-auth login <secret>` to store the PAT in the OS-correct location.
#
# No browser. No manual copy/paste.
#
# Usage:
#   scripts/cowork-headless-login.sh [--name laptop-wsl] [--gateway http://localhost:8080] [--helper path]

set -euo pipefail

PROFILE="${PROFILE:-local}"
GATEWAY="${GATEWAY:-http://localhost:8080}"
PAT_NAME="${PAT_NAME:-$(hostname)-$(date +%s)}"
HELPER="${HELPER:-}"
ADMIN_EMAIL="${SYSTEMPROMPT_ADMIN_EMAIL:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --name) PAT_NAME="$2"; shift 2 ;;
    --gateway) GATEWAY="$2"; shift 2 ;;
    --helper) HELPER="$2"; shift 2 ;;
    --profile) PROFILE="$2"; shift 2 ;;
    --email) ADMIN_EMAIL="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 64 ;;
  esac
done

if [[ -z "$HELPER" ]]; then
  for cand in \
    ./target/release/sp-cowork-auth \
    ../systemprompt-core/bin/cowork-auth/target/release/sp-cowork-auth \
    "$(command -v sp-cowork-auth 2>/dev/null || true)"; do
    if [[ -x "$cand" ]]; then HELPER="$cand"; break; fi
  done
fi
if [[ -z "$HELPER" || ! -x "$HELPER" ]]; then
  echo "sp-cowork-auth not found. Pass --helper <path> or build it in systemprompt-core/bin/cowork-auth/." >&2
  exit 2
fi

SESSIONS="./.systemprompt/sessions/index.json"
JWT=""

json_get() {
  python3 -c "import sys,json;d=json.load(sys.stdin);print($1)" 2>/dev/null || true
}

# 1a. Try the existing session file.
if [[ -f "$SESSIONS" ]]; then
  JWT="$(cat "$SESSIONS" | json_get "d['sessions'].get('$PROFILE',{}).get('session_token','')")"
  if [[ -n "$JWT" ]]; then
    NOW=$(date +%s)
    EXP="$(printf '%s' "$JWT" | awk -F. '{print $2}' | tr '_-' '/+' | (base64 -d 2>/dev/null || true) | json_get "d.get('exp',0)")"
    if [[ -z "$EXP" || "$EXP" -le "$((NOW + 30))" ]]; then
      echo "[cowork-headless] stored CLI session is stale — refreshing"
      JWT=""
    fi
  fi
fi

# 1b. Refresh via CLI if needed.
if [[ -z "$JWT" ]]; then
  echo "[cowork-headless] fetching fresh CLI session JWT"
  if [[ -n "$ADMIN_EMAIL" ]]; then
    JWT=$(SYSTEMPROMPT_ADMIN_EMAIL="$ADMIN_EMAIL" \
      systemprompt admin session login --profile "$PROFILE" --token-only 2>/dev/null | tail -1)
  else
    JWT=$(systemprompt admin session login --profile "$PROFILE" --token-only 2>/dev/null | tail -1)
  fi
fi

if [[ -z "$JWT" || "$JWT" != eyJ* ]]; then
  echo "[cowork-headless] failed to obtain an admin JWT." >&2
  echo "  Try: SYSTEMPROMPT_ADMIN_EMAIL=<email> $0 ..." >&2
  exit 3
fi

# 2. Mint PAT.
echo "[cowork-headless] minting PAT '$PAT_NAME' via $GATEWAY"
RESP="$(curl -sS --fail-with-body -X POST "$GATEWAY/admin/devices/pats" \
  -H "Authorization: Bearer $JWT" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"$PAT_NAME\"}")" || {
  echo "[cowork-headless] PAT mint failed" >&2
  echo "$RESP" >&2
  exit 4
}

SECRET="$(printf '%s' "$RESP" | json_get "d['secret']")"
if [[ -z "$SECRET" || "$SECRET" != sp-live-* ]]; then
  echo "[cowork-headless] gateway did not return a PAT secret" >&2
  echo "$RESP" >&2
  exit 5
fi

# 3. Wire it into the helper.
"$HELPER" login "$SECRET" --gateway "$GATEWAY"

echo "[cowork-headless] done. Test with: $HELPER"
