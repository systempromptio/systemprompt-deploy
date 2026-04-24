# Install via Scoop (Windows)

[Scoop](https://scoop.sh) is a user-mode Windows package manager. Uses the [`systempromptio/scoop-bucket`](https://github.com/systempromptio/scoop-bucket) bucket.

## Install Scoop (once)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression
```

## Install gateway

```powershell
scoop bucket add systemprompt https://github.com/systempromptio/scoop-bucket
scoop install gateway
```

## Upgrade

```powershell
scoop update gateway
```

## Run as a Windows service

```powershell
scoop install nssm
nssm install systemprompt "$(scoop which systemprompt)"
nssm set systemprompt AppEnvironmentExtra "DATABASE_URL=postgres://..." "ANTHROPIC_API_KEY=sk-ant-..."
nssm start systemprompt
```

## Uninstall

```powershell
scoop uninstall gateway
```

Docs: https://systemprompt.io/documentation/?utm_source=scoop&utm_medium=install_doc
