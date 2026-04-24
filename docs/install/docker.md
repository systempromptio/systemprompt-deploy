# Install via Docker Hub

`systemprompt` is published to Docker Hub as [`systemprompt/gateway`](https://hub.docker.com/r/systemprompt/gateway) — multi-arch (`linux/amd64`, `linux/arm64`), cosign-signed, SBOM-attested.

## Quickstart

```bash
docker run --rm -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pw@host:5432/systemprompt \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  systemprompt/gateway:latest
```

Healthcheck: `curl http://localhost:8080/api/v1/health`.

## Compose (Postgres included)

```yaml
services:
  postgres:
    image: postgres:18-alpine
    environment:
      POSTGRES_USER: systemprompt
      POSTGRES_PASSWORD: systemprompt
      POSTGRES_DB: systemprompt
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U systemprompt -d systemprompt"]
      interval: 5s
      retries: 10

  gateway:
    image: systemprompt/gateway:latest
    depends_on:
      postgres:
        condition: service_healthy
    environment:
      DATABASE_URL: postgres://systemprompt:systemprompt@postgres:5432/systemprompt
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
    ports:
      - "8080:8080"
```

## Tags

| Tag | Meaning |
|---|---|
| `latest` | Most recent stable release |
| `0` | Latest `0.x` |
| `0.2` | Latest `0.2.x` |
| `0.2.2` | Exact version (immutable) |

Pin to an exact version in production.

## Verify signature

```bash
cosign verify \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-template/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  systemprompt/gateway:0.2.2
```

View the attached SBOM:

```bash
cosign download attestation systemprompt/gateway:0.2.2 | jq -r '.payload | @base64d | fromjson | .predicate'
```

## Environment variables

| Var | Required | Default |
|---|---|---|
| `DATABASE_URL` | yes | — |
| `ANTHROPIC_API_KEY` | at least one AI key | — |
| `OPENAI_API_KEY` | at least one AI key | — |
| `GEMINI_API_KEY` | at least one AI key | — |
| `HOST` | no | `0.0.0.0` |
| `PORT` | no | `8080` |
| `RUST_LOG` | no | `info` |

## Links

- Source: https://github.com/systempromptio/systemprompt-template
- Docs: https://systemprompt.io/documentation/?utm_source=dockerhub&utm_medium=install_doc
- Licence: `MIT AND BUSL-1.1` — template code is [MIT](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE); the compiled binary links `systemprompt-core` which is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE) (converts to Apache 2.0 after 4 years; production use of the compiled image requires a commercial core licence).
