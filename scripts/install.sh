#!/usr/bin/env bash
# systemprompt installer — https://get.systemprompt.io
#
# Usage:   curl -sSL get.systemprompt.io | sh
#          curl -sSL get.systemprompt.io | sh -s -- --version 0.3.0
#          curl -sSL get.systemprompt.io | sh -s -- --prefix /usr/local --verify
#
# Detects OS + arch, downloads the matching tarball from GitHub Releases,
# verifies SHA256, optionally cosign-verifies, and installs the binary.

set -euo pipefail

REPO="systempromptio/systemprompt-template"
BIN_NAME="systemprompt"
VERSION="latest"
PREFIX=""
VERIFY_COSIGN="false"

log()  { printf '\033[0;36m[install]\033[0m %s\n' "$*" >&2; }
warn() { printf '\033[0;33m[install]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[0;31m[install] error:\033[0m %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --prefix)  PREFIX="$2";  shift 2 ;;
    --verify)  VERIFY_COSIGN="true"; shift ;;
    -h|--help)
      sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) die "unknown flag: $1" ;;
  esac
done

need() { command -v "$1" >/dev/null 2>&1 || die "missing required tool: $1"; }
need curl
need tar
need uname

uname_s=$(uname -s | tr '[:upper:]' '[:lower:]')
uname_m=$(uname -m)

case "$uname_s" in
  linux)  os="linux" ;;
  darwin) os="darwin" ;;
  msys*|mingw*|cygwin*) die "Windows — use Scoop or winget (see docs/install/scoop.md or docs/install/winget.md)" ;;
  *) die "unsupported OS: $uname_s" ;;
esac

case "$uname_m" in
  x86_64|amd64) arch="amd64" ;;
  arm64|aarch64) arch="arm64" ;;
  *) die "unsupported arch: $uname_m" ;;
esac

target="${os}-${arch}"

if [ "$VERSION" = "latest" ]; then
  log "resolving latest release..."
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep -oE '"tag_name"\s*:\s*"[^"]+"' \
    | head -n1 \
    | sed -E 's/.*"([^"]+)"$/\1/')
  [ -n "$VERSION" ] || die "could not resolve latest release"
fi

log "installing ${BIN_NAME} ${VERSION} for ${target}"

tarball="${BIN_NAME}-${VERSION}-${target}.tar.gz"
base="https://github.com/${REPO}/releases/download/${VERSION}"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

log "downloading ${tarball}..."
curl -fsSL "${base}/${tarball}" -o "${tmp}/${tarball}"
curl -fsSL "${base}/SHA256SUMS" -o "${tmp}/SHA256SUMS"

log "verifying SHA256..."
(cd "$tmp" && grep " ${tarball}$" SHA256SUMS | sha256sum -c -)

if [ "$VERIFY_COSIGN" = "true" ]; then
  need cosign
  log "verifying cosign signature..."
  curl -fsSL "${base}/SHA256SUMS.sig" -o "${tmp}/SHA256SUMS.sig"
  curl -fsSL "${base}/SHA256SUMS.pem" -o "${tmp}/SHA256SUMS.pem"
  cosign verify-blob \
    --certificate-identity-regexp="https://github.com/${REPO}/" \
    --certificate-oidc-issuer="https://token.actions.githubusercontent.com" \
    --signature "${tmp}/SHA256SUMS.sig" \
    --certificate "${tmp}/SHA256SUMS.pem" \
    "${tmp}/SHA256SUMS"
fi

log "extracting..."
tar -xzf "${tmp}/${tarball}" -C "$tmp"

if [ -z "$PREFIX" ]; then
  if [ "$(id -u)" -eq 0 ]; then
    PREFIX="/usr/local"
  else
    PREFIX="${HOME}/.local"
  fi
fi

dest="${PREFIX}/bin"
mkdir -p "$dest"

installed=""
for b in systemprompt systemprompt-mcp-agent systemprompt-mcp-marketplace; do
  if [ -f "${tmp}/${b}" ]; then
    install -m 0755 "${tmp}/${b}" "${dest}/${b}"
    installed="${installed} ${b}"
  fi
done

[ -n "$installed" ] || die "no binaries found in tarball"

log "installed:${installed}"
log "location: ${dest}"

case ":$PATH:" in
  *":${dest}:"*) ;;
  *) warn "add ${dest} to your PATH: export PATH=\"${dest}:\$PATH\"" ;;
esac

log "verify with: ${BIN_NAME} --version"
