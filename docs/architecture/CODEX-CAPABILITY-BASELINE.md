# Codex Capability Baseline

## Frozen observation

This baseline was captured on 2026-07-19 by CAC-01 on Windows x86_64. The command was read-only: it inspected Codex version/help/features/auth status, sanitized context-related config, and the bundled and resolved model catalogs. It did not change Codex configuration and did not send a model request.

The machine-readable report is `docs/evidence/CAC-01/capability-baseline.json` and conforms to `schemas/capability-probe.schema.json`.

## Result

The exact configured and catalog model is `gpt-5.6-sol`, but the active Codex catalog does not expose Sol's documented one-million-class window.

| Layer | Total/context window | Maximum input | Maximum output | Effective percent | Effective Codex budget |
|---|---:|---:|---:|---:|---:|
| Official GPT-5.6 Sol model specification | 1,050,000 | 922,000 | 128,000 | Not a client policy | Not a client policy |
| Codex 0.144.5 bundled catalog | 372,000 | Not separately represented | Not separately represented | 95% | 353,400 |
| Codex 0.144.5 resolved catalog | 272,000 | Not separately represented | Not separately represented | 95% | 258,400 |
| Current open-source Codex `main` catalog | 272,000 | Not separately represented | Not separately represented | Codex default applies | 258,400 when 95% applies |

The user's “256k class” experience is reproduced by the resolved catalog: `272,000 × 95% = 258,400` effective tokens. The discrepancy is therefore real and layered:

1. Sol documents a 1,050,000-token total model window.
2. The installed binary carries a 372,000-token bundled Sol record.
3. Codex's resolved runtime catalog replaces that with a 272,000-token Sol record.
4. Codex reserves five percent, leaving 258,400 effective tokens.

Neither the 372,000 bundled value nor the 272,000 resolved value meets the project gate of at least 1,000,000 internal Codex tokens. CAC-01 therefore records `native_one_million_gate_ready = false`. This is a baseline failure to be fixed, not a goal reduction.

## Installed runtime

- Codex: `codex-cli 0.144.5`
- Authentication lane: ChatGPT; authenticated
- Root configured model: exact `gpt-5.6-sol`
- Explicit `model_context_window`: absent
- Explicit `model_auto_compact_token_limit`: absent
- Explicit `model_catalog_json`: absent
- Profile model overrides detected: zero
- Hook feature: available, stable, enabled
- Plugin feature: available, stable, enabled
- `plugin` and `mcp` commands: present
- `use_responses_lite` on both observed Sol entries: true

Only the listed config facts are retained. Arbitrary config keys, credential material, model instructions, raw catalog contents, and authentication output are excluded from evidence.

## Catalog drift finding

The bundled and resolved catalogs are deliberately captured separately because they are not identical:

| Catalog | Normalized SHA-256 | Models | Sol context/max |
|---|---|---:|---:|
| Bundled | `eac5d88049773fc599b5581f54c9f1ff22381401b7ea4c935994556afd0e7d2e` | 8 | 372,000 / 372,000 |
| Resolved | `a678f0497f84fe926476bc2e0f8f760c6a8fd8156de6c690e43c2fe245207b9a` | 8 | 272,000 / 272,000 |

The current open-source catalog was frozen at commit `c86b1be3cdbe12307843bcc9e7a44c1904ddcdf1` and model-catalog blob `a43af2a54ed82719b011f6e8498f9028f340a5ce`. Its Sol entry is 272,000 / 272,000 with `use_responses_lite = true`.

This three-way observation supersedes the planning shorthand that described the “installed catalog” as simply 272,000. The installed binary's bundled data is 372,000; the runtime-resolved and current open-source catalogs are 272,000. Future probes must preserve this distinction and fail closed on unrecognized drift.

## Reproduction

From the repository root:

```powershell
cargo run --locked -- probe --output docs/evidence/CAC-01/capability-baseline.json
```

The command invokes only local Codex diagnostics:

- `codex --version`
- `codex --help`
- `codex features list`
- `codex login status`
- `codex debug models --bundled`
- `codex debug models`

It never invokes `codex exec` or another model-calling command.

## Authoritative sources

- [GPT-5.6 Sol model limits](https://developers.openai.com/api/docs/models/gpt-5.6-sol)
- [Pinned open-source Codex model catalog](https://github.com/openai/codex/blob/c86b1be3cdbe12307843bcc9e7a44c1904ddcdf1/codex-rs/models-manager/models.json)
- [Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
- [Codex hooks](https://learn.chatgpt.com/docs/hooks)
- [Codex plugin building](https://learn.chatgpt.com/docs/build-plugins)

The official Sol page also states that prompts above 272,000 input tokens receive premium pricing. That fact is recorded for later cost gates; CAC-01 performs no paid probe.
