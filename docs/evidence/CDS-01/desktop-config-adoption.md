# CDS-01 desktop configuration adoption evidence

## Verdict

Passed for host configuration adoption on 2026-07-20. A newly created task in the installed Codex Desktop host ran exact `gpt-5.6-sol` and emitted an authoritative effective context window of `1008000`, which is 96 percent of the configured `1050000` catalog window.

This prompt did not run a long-context request or prove compaction behavior at the boundary. Those claims remain gated by G2 and G3.

## Host and configuration contract

- Installed package after the Microsoft Store update observed during the test: `OpenAI.Codex_26.715.7063.0_x64__2p2nqsd0c76g0`.
- Desktop app-server reported by the disposable task: `0.145.0-alpha.18`.
- Exact model: `gpt-5.6-sol`.
- Total catalog window: `1050000`.
- Effective Codex window: `1008000` at the catalog's 96 percent factor.
- Automatic-compaction setting loaded at startup: `900000` with scope `total`.
- Official shared configuration keys used: `model_context_window`, `model_auto_compact_token_limit`, `model_auto_compact_token_limit_scope`, and `model_catalog_json`.

The installed app-server protocol schema defines the window and compaction values as nullable 64-bit integers, defines compaction scope as `total` or `body_after_prefix`, and exposes `config/read`. The catalog path is startup-only, so the packaged app-server was restarted before the disposable task.

## Isolated proof before profile mutation

The existing release helper generated a disposable exact-Sol catalog under the writable sidecar workspace and asked Codex 0.144.5 to resolve it with command-scoped overrides. Codex returned:

```text
model: gpt-5.6-sol
context_window: 1050000
max_context_window: 1050000
effective_context_window_percent: 96
effective_codex_budget: 1008000
auto_compact_token_limit: 900000
catalog_sha256: aa0044cb49656689bffdf55c0334826118a9afcb5cee29fabebe3e0b356b6707
global_config_changed: false
```

A no-request app-server `config/read` call with the same command-scoped settings returned the exact model, `1050000`, `900000`, `total`, and the isolated catalog path.

## Real Desktop acceptance

- Disposable task: `019f7fd8-2c21-7753-a4bf-6b8988326e8f`.
- Task originator: `Codex Desktop`.
- Task source: `vscode` (the installed Desktop host's current protocol label).
- Task result: `CDS-01 OK`.
- Active model: `gpt-5.6-sol`.
- Host-authoritative `model_context_window`: `1008000`.
- First-turn token total: `25444`.
- Compaction events: `0`.
- Local rollout: `C:\Users\17076\.codex\sessions\2026\07\20\rollout-2026-07-20T07-05-06-019f7fd8-2c21-7753-a4bf-6b8988326e8f.jsonl`.

The rollout does not echo the configured automatic-compaction threshold per thread. Threshold adoption is therefore established at the supported startup configuration layer, while actual at-boundary behavior remains unclaimed until G3.

## Mutation and rollback record

The user separately approved the temporary profile change and Desktop restart/control.

- Pre-test config SHA-256: `439752EEBF9FE64D6E1C4A6AA9A2459555C89839ED0DB8E26BE219611625306E`.
- Initial temporary installed SHA-256: `CFCA44DF6A31E2EFD51CC4DC5CC6B3B31D81C95A0EA8B036E418109F0344D709`.
- The Store update changed five app-owned marketplace/browser runtime values during restart.
- Final restored SHA-256: `90329A7D57F48C5472E5357FAD5906C37EEE5869C0FAA72DB0AEA4E6D8D1C80F`.
- Final count of temporary owned keys: `0`.
- The final hash intentionally differs from the pre-test hash because rollback preserved the Store update's later app-owned values instead of overwriting them.
- A replacement packaged app-server was verified running after rollback.
- The disposable acceptance task was archived after evidence capture.

No installed binary was modified, no credential was copied or displayed, no long-context request was sent, and no remote state changed.

## References

- [Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
- [Codex app-server protocol](https://developers.openai.com/codex/app-server)
