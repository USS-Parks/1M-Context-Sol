# Atomic Codex configuration management

CAC-11 adds a reversible manager for the small, declared part of Codex configuration that Context Continuum needs. It does not install the product and never assumes a real user path. Every `cctx config` command requires explicit absolute paths.

## Owned settings

The ownership manifest names the exact managed surface:

- exact root model `gpt-5.6-sol`;
- `model_context_window = 1050000`;
- optional paired `model_auto_compact_token_limit` and `model_auto_compact_token_limit_scope` values;
- the generated `model_catalog_json` path;
- `features.hooks` and `features.plugins` enablement;
- the `mcp_servers.context_continuum` command, `mcp serve` arguments, enablement, required status, and optional startup timeout.

The manager will not adopt or replace a pre-existing `mcp_servers.context_continuum` table unless an active ownership manifest proves that Context Continuum created it. Other tables, values, comments, ordering, and formatting are carried through by the comment-preserving TOML editor. The dry-run JSON contains only changed owned leaves and whole-file hashes; it never echoes unrelated configuration.

Codex's current configuration reference defines the model window, model catalog, compaction-limit, feature, and MCP keys. Its hook reference describes `SessionStart`, `PreCompact`, `PostCompact`, and plugin-bundled hooks. CAC-11 only manages the configuration seams; later roster prompts implement and validate the corresponding runtime behavior.

- [Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
- [Codex hooks reference](https://learn.chatgpt.com/docs/hooks)

## Plan, apply, and rollback

```text
plan:       read exact config + manifest bytes -> validate ownership -> render safe diff
apply:      lock -> re-read exact bytes -> backup/snapshot -> pending manifest -> atomic config -> installed manifest
uninstall:  lock -> require exact installed snapshot -> restore exact backup or remove newly created file -> uninstalled manifest
```

`plan` is read-only. `apply` refuses if either the configuration or ownership manifest changed after planning. Before its first write, it records the exact original bytes in a timestamped backup when the file existed, stores the exact candidate bytes in an installed snapshot, and writes a `pending_install` manifest. It then uses a same-directory atomic replacement for the configuration and advances the manifest to `installed`.

A managed update is accepted only when the current configuration still equals the last installed snapshot. The update retains the first installation's backup and pre-install hash, so the final uninstall restores the bytes from before Context Continuum first took ownership.

`restore` and `uninstall` are equivalent guarded operations. They compare the current configuration, installed snapshot, and manifest hash. Any later user edit, missing file, corrupted snapshot, corrupted backup, ownership-path drift, or pending lifecycle causes a fail-closed refusal. When the original configuration existed, rollback writes its exact bytes atomically. When it did not exist, rollback removes only the byte-identical file recorded as Context Continuum's installed output.

## State layout

```text
<state-dir>/
  config-manager.lock
  ownership.json
  backups/original-unix-<seconds>-<nanos>-<hash>.toml
  snapshots/installed-unix-<seconds>-<nanos>-<hash>.toml
```

The ownership document conforms to [`schemas/config-ownership-manifest.schema.json`](../../schemas/config-ownership-manifest.schema.json). State and direct file symlinks are rejected. A `pending_install` state is intentionally not guessed through: the command stops with manual-recovery guidance so it cannot overwrite ambiguous user data.

## Explicit command surface

```powershell
cctx config plan --config <absolute-config> --state-dir <absolute-state> --catalog <absolute-catalog> --cctx <absolute-cctx>
cctx config apply --config <absolute-config> --state-dir <absolute-state> --catalog <absolute-catalog> --cctx <absolute-cctx>
cctx config restore --config <absolute-config> --state-dir <absolute-state>
cctx config uninstall --config <absolute-config> --state-dir <absolute-state>
```

An automatic-compaction limit is omitted at CAC-11. A caller may exercise the already validated paired fields with `--auto-compact-token-limit` and `--auto-compact-scope`, but CAC-14 remains the authority for the calibrated product value. The real global Codex installation remains outside this prompt and requires the named CAC-45 approval gate.
