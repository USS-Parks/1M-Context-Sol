# Headless Linux and Paseo

The Linux lane installs and verifies the GPT-5.6 Sol 1M catalog policy for Codex CLI and fresh Codex agents launched by Paseo. It does not install a ticker, desktop overlay, prompt proxy, replacement TUI, MCP model selector, or alternate API route.

## Supported host profile

- Linux x86_64 with Python 3.11 or newer and Bash.
- Exact `codex-cli 0.145.0`.
- A writable Codex home owned by the account that launches Codex/Paseo.
- A top-level user-owned `model = "gpt-5.6-sol"` already present in `config.toml`.

`CODEX_HOME` is honored when explicitly set and otherwise defaults to `~/.codex`. It must be an existing absolute, non-symlink directory. Run the manager as the same Unix account and with the same `CODEX_HOME` used by the Paseo daemon. Worktrees need no special handling because the managed files remain in Codex user state outside every worktree.

Codex versions other than 0.145.0 fail closed. Supporting a newer version requires a reviewed schema profile and tests; the manager never reuses the frozen 0.144.5 catalog entry.

## Plan and install

From the repository root:

```bash
bash scripts/linux/manage-sol-policy plan
bash scripts/linux/manage-sol-policy install
```

`plan` calls `codex --version` and `codex debug models --bundled`, but writes no files and sends no model request. It prints the exact config/catalog/backup/manifest paths, hashes, owned keys, and candidate values.

`install` captures the installed 0.145.0 bundled catalog, requires exactly one schema-compatible `gpt-5.6-sol` entry, preserves that entry, and changes only:

| Catalog field | Installed value |
|---|---:|
| `context_window` | 1,050,000 |
| `max_context_window` | 1,050,000 |
| `auto_compact_token_limit` | 900,000 |

It writes a deterministic one-model catalog under `$CODEX_HOME/sol-1m-linux/`, records source/output/preserved hashes and the changed-field allowlist, saves the original config bytes, and appends exactly these owned top-level keys:

```toml
model_context_window = 1050000
model_auto_compact_token_limit = 900000
model_auto_compact_token_limit_scope = "total"
model_catalog_json = "/absolute/CODEX_HOME/sol-1m-linux/generated-sol-1m-models.json"
```

The manager does not own or rewrite `model`, authentication, provider, approval, sandbox, MCP, or Paseo settings. An existing owned key is a conflict, and repeated installation is refused.

## Status and verification

```bash
bash scripts/linux/manage-sol-policy status
bash scripts/linux/manage-sol-policy verify
```

`status` reports the installed manager/Codex versions, whole-config snapshot state, owned-value drift, user-model state, generated-catalog integrity, installed source-catalog compatibility, and the latest verified host budget when one was actually observed. It sends no model request.

`verify` loads the installed catalog through `codex debug models`, proves exact Sol resolution and non-policy-field preservation, and writes a local verification record. It sends no model request unless the user adds `--live`:

```bash
bash scripts/linux/manage-sol-policy verify --live
```

`--live` explicitly permits one fresh, ephemeral, read-only `gpt-5.6-sol` Codex task. The manager records a host budget only if Codex emits one in its JSONL task metadata; successful catalog verification does not invent a missing live budget.

## Launch through Paseo

The `model_catalog_json` key is consumed at Codex/app-server startup. After installation, create a fresh Codex app-server/agent; an already-running agent is not proof that the new catalog was loaded.

Use the saved Codex model:

```bash
paseo run --provider codex "your task"
```

Or select Sol explicitly:

```bash
paseo run --provider codex/gpt-5.6-sol "your task"
```

An explicit different Paseo model remains authoritative. For example, `--provider codex/gpt-5.4` does not use the Sol policy and is never silently replaced.

The manager never stops or restarts the Paseo daemon. A daemon restart interrupts active agents and remains a user-operated maintenance decision. Starting a fresh agent is the required policy reload boundary.

## Uninstall

```bash
bash scripts/linux/manage-sol-policy uninstall
```

If the installed config is unchanged, uninstall restores the original bytes exactly. If unrelated keys changed later, it removes only the unchanged managed block and preserves those edits. If any owned key or the managed block changed, uninstall refuses to remove anything. A fresh Codex app-server/agent is required after uninstall; Paseo is not restarted automatically.

## Fail-closed boundaries

Generation or installation stops on an unsupported Codex version, unknown root/Sol schema field, missing field, wrong field type, duplicate slug, missing/duplicate Sol entry, invalid TOML/JSON, non-exact user model, existing owned key, symlinked state path, or source/output preservation failure. Install, plan, and status never send a model request.
