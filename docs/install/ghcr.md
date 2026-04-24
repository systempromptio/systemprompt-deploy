# Install via GitHub Container Registry

`systemprompt` is published to GHCR as [`ghcr.io/systempromptio/systemprompt-template`](https://github.com/systempromptio/systemprompt-template/pkgs/container/systemprompt-template) — identical image to [Docker Hub](docker.md) but with no anonymous pull-rate limits and GitHub-native auth.

Pick GHCR when:
- You're running in a network that hits Docker Hub's anonymous pull limit (100 per 6 hours).
- You already authenticate to `ghcr.io` for private packages.
- You prefer pulling from the same host as the source repo.

## Quickstart

```bash
docker run --rm -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pw@host:5432/systemprompt \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  ghcr.io/systempromptio/systemprompt-template:latest
```

## Tags

Same scheme as Docker Hub: `latest`, `0`, `0.2`, `0.2.2`.

## Authenticated pulls (if rate-limited or private)

```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u <your-github-username> --password-stdin
```

The `GITHUB_TOKEN` needs `read:packages`.

## Verify signature

```bash
cosign verify \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-template/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  ghcr.io/systempromptio/systemprompt-template:0.2.2
```

## Everything else

See [docker.md](docker.md) for Compose, environment variables, and links — the image is the same.

Docs: https://systemprompt.io/documentation/?utm_source=ghcr&utm_medium=install_doc
