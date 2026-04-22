#!/bin/bash
# DEMO: COWORK DEVICE ROUNDTRIP — SESSION FLOW
# Exercises the server-side half of the sp-cowork-auth session-exchange flow
# without requiring a real browser: mint an exchange code via CLI, POST it to
# the gateway, assert a JWT + canonical header envelope come back.
#
# What this does:
#   1. Picks an existing user from the local tenant.
#   2. Issues a one-shot session exchange code via CLI.
#   3. POSTs the code to /v1/auth/cowork/session.
#   4. Asserts the response contains token + x-user-id header.
#
# mTLS mode is skipped — needs a real device certificate enrolled first
# (see: systemprompt admin cowork enroll-cert).
#
# PAT mode is skipped — it is covered by the /admin/devices UI; CLI-side
# PAT issuance is not yet wired.
#
# Cost: Free (no AI call)

set -e

source "$(cd "$(dirname "$0")/.." && pwd)/_common.sh"

header "COWORK: DEVICE ROUNDTRIP" "Session exchange flow end-to-end"

subheader "STEP 1: Resolve a user id"
USER_JSON="$("$CLI" admin users list --profile "$PROFILE" 2>/dev/null | sed -n '/^{/,$p')"
USER_ID="$(printf '%s' "$USER_JSON" | sed -n 's/.*"id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
if [[ -z "$USER_ID" ]]; then
  fail "no user found — run ./demo/00-preflight.sh first"
  exit 1
fi
pass "user_id=$USER_ID"

subheader "STEP 2: Issue exchange code"
cmd "systemprompt admin cowork issue-code --user-id $USER_ID"
ISSUE_OUT="$("$CLI" admin cowork issue-code --user-id "$USER_ID" --profile "$PROFILE" 2>&1)"
CODE="$(printf '%s' "$ISSUE_OUT" | sed -n 's/.*"code"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
if [[ -z "$CODE" ]]; then
  fail "exchange code not returned"
  printf '%s\n' "$ISSUE_OUT"
  exit 1
fi
pass "code length=${#CODE} (expected 64 hex chars)"

subheader "STEP 3: Exchange via gateway"
EXCHANGE_URL="${BASE_URL}/v1/auth/cowork/session"
cmd "curl -s -X POST $EXCHANGE_URL -H 'Content-Type: application/json' -d '{\"code\":\"***\"}'"
RESP="$(curl -s -o /tmp/cowork_resp.json -w '%{http_code}' -X POST "$EXCHANGE_URL" \
  -H 'Content-Type: application/json' \
  -d "{\"code\":\"$CODE\"}")"
BODY="$(cat /tmp/cowork_resp.json 2>/dev/null || echo '')"
rm -f /tmp/cowork_resp.json

if [[ "$RESP" != "200" ]]; then
  fail "gateway returned HTTP $RESP"
  printf '%s\n' "$BODY"
  exit 1
fi
pass "HTTP 200 from gateway"

subheader "STEP 4: Assert token + canonical headers"
TOKEN="$(printf '%s' "$BODY" | sed -n 's/.*"token"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
UID_HDR="$(printf '%s' "$BODY" | sed -n 's/.*"x-user-id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"

if [[ -z "$TOKEN" ]]; then
  fail "token missing from response"
  printf '%s\n' "$BODY"
  exit 1
fi
pass "token present (length=${#TOKEN})"

if [[ "$UID_HDR" != "$USER_ID" ]]; then
  fail "x-user-id mismatch (expected=$USER_ID got=$UID_HDR)"
  printf '%s\n' "$BODY"
  exit 1
fi
pass "x-user-id matches issued user"

subheader "STEP 5: Replay should fail"
REPLAY="$(curl -s -o /dev/null -w '%{http_code}' -X POST "$EXCHANGE_URL" \
  -H 'Content-Type: application/json' \
  -d "{\"code\":\"$CODE\"}")"
if [[ "$REPLAY" == "200" ]]; then
  fail "replay succeeded — exchange code should be single-use"
  exit 1
fi
pass "replay rejected (HTTP $REPLAY)"

header "COWORK DEVICE ROUNDTRIP COMPLETE"
info "Next: run 'sp-cowork-auth session' on a workstation to exercise the browser half."
