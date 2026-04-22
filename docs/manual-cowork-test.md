# Manual Cowork Device Auth Testing

End-to-end manual test plan for the `sp-cowork-auth` CLI against a locally-running template. Exercises all three authentication modes (`pat`, `session`, `mtls`) plus the consent page, the devices UI, and the negative paths.

**Audience:** engineer verifying a build before merging or before tagging a release.

**Estimated time:** 10–15 minutes.

---

## 0. Prerequisites

```bash
# From the template repo root:
just build               # compiles template + patched core 0.3.0
just start &             # starts services on http://localhost:8080
```

Wait until you see `✓ All services started successfully` in the log.

Sanity:

```bash
curl -s http://localhost:8080/v1/auth/cowork/capabilities
# {"modes":["pat","session","mtls"]}
```

Build the helper binary (sibling repo):

```bash
cd ../systemprompt-core/bin/cowork-auth
cargo build --release
HELPER="$(pwd)/target/release/sp-cowork-auth"
$HELPER help
```

Expected help output lists: `login`, `logout`, `status`, `run`, `help`.

---

## 1. Log in to the dashboard (required for every flow)

Open `http://localhost:8080/admin/login` in a browser and sign in with your seed user.

**The dashboard cookie is the single source of identity.** The consent page (session flow) and the devices UI (PAT/cert management) both check this cookie; the `sp-cowork-auth` CLI does not share it.

Verify:

- Visit `http://localhost:8080/admin/dashboard` — should render.
- Visit `http://localhost:8080/cowork-auth/setup` — should render the "Connect Claude" page with your email in the callout.
- Visit `http://localhost:8080/admin/devices` — should render empty PAT / cert tables.

---

## 2. Automated baseline (no browser)

Before doing anything manual, confirm the server-side half is healthy:

```bash
cd /var/www/html/systemprompt-template
./demo/users/05-cowork-device-roundtrip.sh
```

Expected: 5/5 `✓`. If this fails, stop — the server build is broken and no manual flow will work.

---

## How the helper CLI is invoked

`sp-cowork-auth` has no per-mode subcommand for authentication. The single `run` command (also the default when no args are given) probes providers in order **mtls → session → pat** and prints the first successful JWT envelope to stdout. You pick a mode by configuring (or not configuring) each provider in `cowork-auth.toml`.

| Command | Purpose |
|---|---|
| `sp-cowork-auth login <sp-live-...> [--gateway URL]` | Store a PAT + write `cowork-auth.toml`. Secret goes to a 0600 file. |
| `sp-cowork-auth logout` | Remove the PAT file and strip `[pat]` from the config. |
| `sp-cowork-auth status` | Print the config paths and whether each file exists. |
| `sp-cowork-auth run` (or just `sp-cowork-auth`) | Authenticate and emit a JWT envelope. |
| `sp-cowork-auth help` | Show usage. |

Env overrides: `SP_COWORK_CONFIG`, `SP_COWORK_PAT`, `SP_COWORK_GATEWAY_URL`.

---

## 3. PAT flow (simplest, fully scriptable)

### 3.1 Issue a PAT

1. Go to `http://localhost:8080/admin/devices`.
2. In **Personal access tokens (PATs)**, enter a name (e.g. `laptop-test`) and click **Issue PAT**.
3. Copy the `sp-live-…` secret displayed below the form — this is your only chance.

### 3.2 Store the PAT with `login`

One command — writes the config + secret to the OS-correct location with locked-down permissions:

```bash
$HELPER login sp-live-PASTE_YOUR_SECRET --gateway http://localhost:8080
$HELPER status
```

Expected `login` output:

```
Stored PAT for cowork-auth helper.
  config: <config_dir>/systemprompt/cowork-auth.toml
  secret: <config_dir>/systemprompt/cowork-auth.pat (0600)
Next: run `sp-cowork-auth` to fetch a JWT.
```

`<config_dir>` per platform:

| Platform | Path |
|---|---|
| Linux / WSL | `~/.config/systemprompt/` |
| macOS | `~/Library/Application Support/systemprompt/` |
| Windows | `%APPDATA%\systemprompt\` |

Permissions: dir `0700`, secret file `0600`, config file `0644` (Unix). Windows inherits the user-scoped NTFS ACL on `%APPDATA%`.

### 3.3 Validation guards

```bash
$HELPER login "nope"                       # expect: token must start with `sp-live-`
$HELPER login "sp-live-nodotseparator"     # expect: token must contain a `.` separator
$HELPER login "sp-live-short.x"            # expect: token looks too short
```

All three should exit non-zero and leave the config untouched.

### 3.4 Run the helper

```bash
$HELPER
```

Expected (prettified):

```json
{
  "token": "eyJ0eXAi…",
  "ttl": 3600,
  "headers": {
    "x-user-id": "<your-user-id>",
    "x-session-id": "sess_…",
    "x-trace-id": "…",
    "x-tenant-id": "systemprompt-local",
    "x-client-id": "sp_cowork",
    "x-call-source": "cowork",
    "x-policy-version": "unversioned"
  }
}
```

Second invocation within the TTL returns the cached JWT (same `session-id`), printed via the helper's cache layer — no gateway round-trip.

### 3.5 Prove the JWT is accepted downstream

```bash
# Bust the cache first so we get a fresh JWT
rm -f ~/.cache/systemprompt/cowork-auth.json
TOKEN=$($HELPER | python3 -c 'import sys,json;print(json.load(sys.stdin)["token"])')
curl -s -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/core/oauth/userinfo | head -c 300
# should return your user profile JSON
```

### 3.6 Env-only flow (no stored secret)

Useful for CI runners that inject the PAT at runtime:

```bash
$HELPER logout                             # clear the stored secret
export SP_COWORK_PAT="sp-live-..."
rm -f ~/.cache/systemprompt/cowork-auth.json
$HELPER                                    # same JSON envelope, secret never on disk
unset SP_COWORK_PAT
```

### 3.7 Revoke and confirm

1. On `/admin/devices`, click **Revoke** next to the PAT.
2. `rm -f ~/.cache/systemprompt/cowork-auth.json && $HELPER` must now fail.
3. Helper diagnostics should show `pat: …401…` and the process exits `5`.
4. Refresh `/admin/devices` — PAT row shows `revoked` (red).

### 3.8 Empty-name guard on the UI

Attempt to issue a PAT with an empty name via the UI form — the HTML `required` attribute should block submission. Any client that bypasses the attribute should receive HTTP 400 from `POST /admin/devices/pats`.

---

## 4. Session flow (browser consent)

### 4.1 Switch helper config to session mode

The helper probe order is **mtls → session → pat**. With a PAT stored, PAT wins — so clear it first, then enable session:

```bash
$HELPER logout
```

Edit `<config_dir>/systemprompt/cowork-auth.toml` (path from `$HELPER status`):

```toml
gateway_url = "http://localhost:8080"

[session]
enabled = true
```

Confirm:

```bash
$HELPER status
# config_present=true, pat_present=false
```

### 4.2 Run the helper

```bash
rm -f ~/.cache/systemprompt/cowork-auth.json
$HELPER
```

Expected stderr:

```
[sp-cowork-auth] opening browser to http://localhost:8080/cowork-auth/device-link?redirect=http%3A%2F%2F127.0.0.1%3A8767%2Fcallback
```

Helper tries to launch the default browser. **In WSL this often doesn't work** — copy the URL manually into your already-logged-in browser tab. (Install `wslu` and restart the shell if you want `xdg-open` to bridge to the Windows default browser.)

### 4.3 Approve in the browser

Consent page should show:
- Logo + "Authorise a device"
- `A local helper at 127.0.0.1:8767 is asking to sign in as <your-email>.`
- **Allow** / **Deny** buttons.

Click **Allow**.

Browser:
- Redirects to `http://127.0.0.1:8767/callback?code=<64-hex>`.
- Page reads "Authorisation complete — return to your terminal" (or similar from `success.html`).

Terminal:
- Helper prints the JSON envelope described in 3.4.

### 4.4 Replay rejection

Grab the `code` from the browser URL bar. Replay it manually:

```bash
curl -s -o /dev/null -w "%{http_code}\n" -X POST http://localhost:8080/v1/auth/cowork/session \
  -H 'Content-Type: application/json' \
  -d '{"code":"<the-used-code>"}'
# expect: 401 (single-use — code was consumed on first exchange)
```

### 4.5 Deny path

```bash
rm -f ~/.cache/systemprompt/cowork-auth.json
$HELPER
```

Click **Deny** in the browser.
- Browser redirects to `http://127.0.0.1:8767/callback?error=denied`.
- Helper exits non-zero with a `session: dashboard reported error: denied` diagnostic.

### 4.6 Bad-redirect guard (open-redirect defence)

Open directly in the browser while logged in:

```
http://localhost:8080/cowork-auth/device-link?redirect=https%3A%2F%2Fevil.com
```

Expected:
- HTTP 400.
- Body: `Invalid redirect — Only http://127.0.0.1:PORT or http://localhost:PORT redirects are accepted. Got: https://evil.com`.

### 4.7 Unauth redirect preserves full path

Clear cookies for `localhost:8080`, then visit:

```
http://localhost:8080/cowork-auth/device-link?redirect=http%3A%2F%2F127.0.0.1%3A8767%2Fcallback
```

Expected:
- HTTP 307 to `/admin/login?redirect=/cowork-auth/device-link` (full path preserved via `OriginalUri`).
- After logging in, the browser lands back on the consent page with the original `redirect=` intact.

### 4.8 Loopback bind collision

Start a second helper while the first is still waiting for the callback:

```bash
$HELPER &             # first run; leave the browser tab unclicked
$HELPER               # second run — should fail to bind 127.0.0.1:8767
```

Expected second run: `session: bind 127.0.0.1:8767 failed: Address already in use`. Kill the first helper (`fg` then Ctrl-C) before proceeding.

---

## 5. mTLS flow (manual — needs a real cert)

### 5.1 Generate a self-signed device cert

```bash
openssl req -x509 -newkey ed25519 -nodes -days 365 \
  -keyout /tmp/device-key.pem -out /tmp/device-cert.pem \
  -subj "/CN=manual-test-laptop"
FP=$(openssl x509 -in /tmp/device-cert.pem -outform DER | sha256sum | awk '{print $1}')
echo "fingerprint=$FP"
```

### 5.2 Enroll it

```bash
USER_ID=$(systemprompt admin users list --profile local 2>/dev/null | \
  sed -n '/^{/,$p' | \
  sed -n 's/.*"id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)

systemprompt admin cowork enroll-cert \
  --user-id "$USER_ID" \
  --fingerprint "$FP" \
  --label "manual-test-laptop"
```

Verify on `/admin/devices` — the cert row appears under **Enrolled device certificates**.

### 5.3 Configure and run

Clear any PAT first so mTLS is preferred. Edit `<config_dir>/systemprompt/cowork-auth.toml`:

```toml
gateway_url = "http://localhost:8080"

[mtls]
cert_path = "/tmp/device-cert.pem"
```

(macOS keystore: replace with `cert_label = "systemprompt-device"`. Windows store: replace with `cert_sha256 = "<64-hex>"`.)

```bash
$HELPER logout
rm -f ~/.cache/systemprompt/cowork-auth.json
$HELPER
```

Expected: same JWT envelope as other modes; `x-user-id` matches `$USER_ID`.

### 5.4 Fingerprint mismatch

Enroll one cert but point the helper at a different PEM:

```bash
openssl req -x509 -newkey ed25519 -nodes -days 365 \
  -keyout /tmp/device-key-other.pem -out /tmp/device-cert-other.pem \
  -subj "/CN=unenrolled"
# Edit cowork-auth.toml: cert_path = "/tmp/device-cert-other.pem"
rm -f ~/.cache/systemprompt/cowork-auth.json
$HELPER
# expect: mtls: …401… "device certificate not enrolled or revoked"
```

### 5.5 Revoke and retry

Click **Revoke** on the cert row in `/admin/devices`, then point the helper back at the original (now revoked) cert:

```bash
rm -f ~/.cache/systemprompt/cowork-auth.json
$HELPER
# expect: mtls failure; helper exits 5
```

---

## 6. Setup-page sanity (`/cowork-auth/setup`)

Visit `http://localhost:8080/cowork-auth/setup` while logged in. Verify:

1. Callout at the top shows your email and the single-login explainer.
2. **Server capabilities** grid loads `pat`, `session`, `mtls` with green `supported` badges (live from `/v1/auth/cowork/capabilities`).
3. The **login command** snippet shows `sp-cowork-auth login sp-live-YOUR_SECRET --gateway http://localhost:8080` — the **Copy** button next to it works.
4. Expand **Manual config (advanced)** — TOML snippet has the same `gateway_url`; its **Copy** button works.
5. Sidebar shows **Connect Claude** (active) above **Devices** under the **Account** heading.

Visit while logged out: should bounce to `/admin/login?redirect=/cowork-auth/setup`, then back after login.

---

## 7. Cleanup

```bash
# Revoke any PATs or certs you issued from /admin/devices (UI)
$HELPER logout                                 # removes PAT file + strips [pat] section
rm -f ~/.cache/systemprompt/cowork-auth.json   # cached JWT if present
rm -f /tmp/device-cert.pem /tmp/device-key.pem /tmp/device-cert-other.pem /tmp/device-key-other.pem
# Optional: fully remove the config dir
rm -f ~/.config/systemprompt/cowork-auth.toml
rmdir ~/.config/systemprompt 2>/dev/null
```

---

## 8. Pass criteria

All of the following must be true for the build to ship:

- [ ] `./demo/users/05-cowork-device-roundtrip.sh` — 5/5 `✓`.
- [ ] `sp-cowork-auth login/logout/status` work cross-platform (dir `0700`, secret `0600` on Unix).
- [ ] PAT mode authenticates via stored file **and** via `SP_COWORK_PAT`; revoked PAT is rejected.
- [ ] Session mode round-trips through the browser; replay is rejected; deny path exits cleanly; loopback collision is detected.
- [ ] Bad-redirect returns 400; unauth redirect preserves `/cowork-auth/device-link` in the `redirect` query.
- [ ] mTLS enroll → authenticate → revoke works end-to-end; fingerprint mismatch is rejected.
- [ ] `/cowork-auth/setup` renders with live capabilities, correct gateway URL, working copy buttons.
- [ ] `just clippy` clean; `just build` clean.

---

## Reference: route map

| URL | Method | Auth | Purpose |
|---|---|---|---|
| `/admin/login` | GET | — | The single login. |
| `/cowork-auth/setup` | GET | cookie | Human-facing landing page. |
| `/cowork-auth/device-link` | GET | cookie | Consent page the helper opens. |
| `/cowork-auth/device-link/approve` | POST | cookie | Mint exchange code, redirect to loopback. |
| `/cowork-auth/device-link/deny` | POST | cookie | Redirect to loopback with `?error=denied`. |
| `/admin/devices` | GET | cookie | PAT + cert listing. |
| `/admin/devices/pats` | POST | cookie | Issue PAT. |
| `/admin/devices/pats/{id}` | DELETE | cookie | Revoke PAT. |
| `/admin/devices/certs/{id}` | DELETE | cookie | Revoke enrolled cert. |
| `/v1/auth/cowork/capabilities` | GET | — | Advertises enabled modes. |
| `/v1/auth/cowork/pat` | POST | Bearer PAT | Exchange PAT for JWT. |
| `/v1/auth/cowork/session` | POST | — (code in body) | Exchange one-shot code for JWT. |
| `/v1/auth/cowork/mtls` | POST | — (fingerprint in body) | Exchange cert fingerprint for JWT. |

## Reference: helper file layout

| Path | Purpose | Perms (Unix) |
|---|---|---|
| `<config_dir>/systemprompt/cowork-auth.toml` | Main config (gateway URL + provider blocks) | `0644` |
| `<config_dir>/systemprompt/cowork-auth.pat` | Stored PAT secret (only exists after `login`) | `0600` |
| `<cache_dir>/systemprompt/cowork-auth.json` | Cached JWT envelope until TTL | `0600` |

`<config_dir>` / `<cache_dir>` follow XDG on Linux/WSL (`~/.config`, `~/.cache`), Apple support dirs on macOS (`~/Library/Application Support`, `~/Library/Caches`), and `%APPDATA%` / `%LOCALAPPDATA%` on Windows.
