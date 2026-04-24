# systemprompt-deploy (private)

Production distribution pipelines for systemprompt.io.

This repo is a full fork of [`systemprompt-template`](https://github.com/systempromptio/systemprompt-template) plus the CI/CD that ships binaries, Docker images, Helm chart, Homebrew tap, Scoop bucket, APT/RPM repos to our public channels.

**Users never interact with this repo.** They use the public template + the published channels:

- Docker image: `ghcr.io/systempromptio/systemprompt-template:<tag>`
- Binary install: `curl -sSL https://get.systemprompt.io | sh`
- Helm chart: `helm repo add systemprompt https://charts.systemprompt.io`
- Homebrew: `brew install systempromptio/tap/gateway`
- Scoop: `scoop install systempromptio/gateway`

## Release flow

Tag on this repo → workflows fire:

| Workflow | Publishes to |
|---|---|
| `release.yml` | **Public** `systempromptio/systemprompt-template` Releases (binaries, SHA256SUMS, cosign sig, .deb, .rpm) — via `RELEASE_UPLOAD_TOKEN` |
| `docker.yml` | `ghcr.io/systempromptio/systemprompt-template` — via `GHCR_PUBLISH_TOKEN` |
| `helm.yml` | `systempromptio/charts` gh-pages → `charts.systemprompt.io` — via `CHARTS_REPO_TOKEN` |
| `homebrew.yml` | `systempromptio/homebrew-tap` — via `HOMEBREW_TAP_TOKEN` |
| `scoop.yml` | `systempromptio/scoop-bucket` — via `SCOOP_BUCKET_TOKEN` |
| `apt.yml` | `systempromptio/apt` gh-pages → `deb.systemprompt.io` — **DEFERRED** |
| `rpm.yml` | `systempromptio/rpm` gh-pages → `rpm.systemprompt.io` — **DEFERRED** |
| `winget.yml` | PR to `microsoft/winget-pkgs` — **DEFERRED** |

## Required GitHub Secrets

All fine-grained PATs scoped to the single target repo where possible.

**Critical path (first release):**
- `RELEASE_UPLOAD_TOKEN` — fine-grained PAT on `systempromptio/systemprompt-template`, **Contents: Read and write** (uploads binaries to template's public Releases)
- `GHCR_PUBLISH_TOKEN` — classic PAT, scope `write:packages`, `read:packages` (pushes to GHCR; first push creates the `systemprompt-template` package under systempromptio namespace — set it to public afterwards at https://github.com/orgs/systempromptio/packages)

**Active channels:**
- `HOMEBREW_TAP_TOKEN` — fine-grained, `systempromptio/homebrew-tap`, Contents: Read and write
- `SCOOP_BUCKET_TOKEN` — fine-grained, `systempromptio/scoop-bucket`, Contents: Read and write
- `CHARTS_REPO_TOKEN` — fine-grained, `systempromptio/charts`, Contents: Read and write

**Deferred (add when enabling each):**
- `APT_REPO_TOKEN`, `APT_GPG_PRIVATE_KEY`, `APT_GPG_PASSPHRASE`, `APT_GPG_KEYID`
- `RPM_REPO_TOKEN`, `RPM_GPG_PRIVATE_KEY`, `RPM_GPG_KEYID`
- `WINGET_TOKEN`
- `DOCKERHUB_USERNAME`, `DOCKERHUB_TOKEN` + `DOCKERHUB_ENABLED=true` (Variables tab)

## Adding a secret

https://github.com/systempromptio/systemprompt-deploy/settings/secrets/actions/new

## Syncing from the template

This repo was seeded from `systemprompt-template` at commit `c730c8e`. To pull template improvements later:

```bash
git remote add template https://github.com/systempromptio/systemprompt-template.git
git fetch template main
git merge template/main   # resolve any conflicts around CI workflows (we have extras)
```
