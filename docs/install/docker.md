# Install via Docker Hub

> **Status: not yet published.** Docker Hub requires a paid Team subscription to create an organization, so `docker.io/systemprompt/gateway` is pending.
>
> **Use [GHCR](ghcr.md) instead** — same image, same signing, no pull-rate limits:
> ```bash
> docker run --rm -p 8080:8080 ghcr.io/systempromptio/systemprompt-template:latest
> ```

---

Once `systemprompt/gateway` is live on Docker Hub, the below will apply.

## Quickstart (future state)

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
    image: ghcr.io/systempromptio/systemprompt-template:latest   # swap to systemprompt/gateway when live
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

Once live, images are cosign-signed (keyless OIDC):

```bash
cosign verify \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-template/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  systemprompt/gateway:0.2.2
```

Same command works today against GHCR — just swap the image ref.

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

- **Use GHCR today**: [install/ghcr.md](ghcr.md)
- Source: https://github.com/systempromptio/systemprompt-template
- Docs: https://systemprompt.io/documentation/?utm_source=dockerhub&utm_medium=install_doc
- Licence: `MIT AND BUSL-1.1` — template code is [MIT](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE); the compiled binary links `systemprompt-core` which is [BSL-1.1](https://github.com/systempromptio/systemprompt-core/blob/main/LICENSE) (converts to Apache 2.0 after 4 years; production use of the compiled image requires a commercial core licence).
