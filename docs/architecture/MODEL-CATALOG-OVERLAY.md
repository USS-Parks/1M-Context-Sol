# GPT-5.6 Sol model-catalog overlay

CAC-10 implements a deterministic, version-guarded catalog generator for exact model `gpt-5.6-sol`. It does not install the catalog and it does not prove that the backend accepts an above-cap request. Those claims remain gated by CAC-11 through CAC-16 and G2.

## Supported source profile

The first parser profile is `codex-model-catalog/0.144.5-v1` for `codex-cli 0.144.5`. The parser requires:

- an object root containing only `models`;
- unique string slugs and exactly one `gpt-5.6-sol` entry;
- every required Sol metadata field with its expected JSON type;
- only reviewed optional fields; and
- the current `truncation_policy` object shape.

An unknown Codex version, root property, Sol field, missing required field, duplicate slug, malformed document, or official-limit regression stops generation. A new Codex catalog shape requires a new reviewed parser profile.

## Candidate policy

The CAC-10 candidate changes only these approved fields:

| Field | Candidate | Meaning |
|---|---:|---|
| `context_window` | 1,050,000 | Catalog total-window policy |
| `max_context_window` | 1,050,000 | Maximum allowed config override |
| `effective_context_window_percent` | 96 | 1,008,000-token internal Codex budget |
| `auto_compact_token_limit` | omitted | CAC-14 remains the calibration authority |

The separate official maximum input remains 922,000 and maximum output remains 128,000. The 1,008,000 effective Codex budget is neither a maximum-input claim nor proof that a model request above the old cap succeeds.

Codex derives or clamps automatic compaction to 90% of the active context window. The generator validates an explicitly supplied threshold against that clamp, but the default candidate does not invent one before CAC-14 measurement.

## Preservation contract

Generation clones the installed Sol entry and removes every non-Sol model. It then applies only the four approved policy fields. Before serialization, it removes those four fields from both source and output and requires the remaining JSON objects to be semantically identical.

`base_instructions` string bytes after JSON decoding must remain identical. `model_messages`, tool flags, modality flags, reasoning options, service tiers, compaction compatibility hash, and all other reviewed metadata must remain semantically identical. Output object keys are normalized for deterministic serialization; source formatting and object-key order are not preserved.

## Hash manifest

Each generation emits a Draft 2020-12-validated manifest containing raw and normalized source hashes, source and output Sol hashes, a preserved-metadata hash, the exact changed-field list, the official limits, and the candidate policy. The checked-in [CAC-10 manifest](../evidence/CAC-10/catalog-overlay-manifest.json) was generated from the resolved installed catalog on 2026-07-19. No instruction text or credential is stored in that evidence.

## Read-only generation

```powershell
cargo run --locked -- catalog generate `
  --codex codex `
  --output C:\tmp\sol-overlay.json `
  --manifest C:\tmp\sol-overlay.manifest.json
```

The command runs `codex --version` and `codex debug models`, generates two explicit output files, and sends no model request. File input is also supported with `--input <FILE> --codex-version 0.144.5`.

## Installed-parser compatibility proof

The checked-in implementation generated a one-model catalog from the full resolved installed catalog and loaded it back through Codex with a command-scoped `model_catalog_json` override. Codex reported exactly one `gpt-5.6-sol` entry with policy `1,050,000 / 1,050,000 / 96`; base instruction bytes, model-message semantics, and reviewed feature flags matched the source. The generated catalog SHA-256 was `eceabc60cee218fc5a4bd2042ecccd9330be7986a8095ed26905779a15687081`.

This is local client-parser compatibility evidence only. It is not G2 native-window evidence and makes no release claim.

## Headless Linux profile

The Linux/Paseo manager adds a separate `codex-model-catalog/0.145.0-v1` profile. It captures the version-matched embedded catalog with `codex debug models --bundled`; it does not reuse the checked-in 0.144.5 entry. The profile is pinned to the official `rust-v0.145.0` source tag (`25af12f7e61572b0bc18ddb1008be543b91519b0`) and requires the exact reviewed Sol field set and value kinds.

Codex 0.145.0's source entry differs materially from 0.144.5: its catalog window is 272,000, it has no `effective_context_window_percent` field, and it carries different instruction/capability metadata. Linux generation therefore preserves the complete 0.145.0 source entry and changes only `context_window`, `max_context_window`, and `auto_compact_token_limit`. The separate top-level `model_context_window`, `model_auto_compact_token_limit`, and `model_auto_compact_token_limit_scope` keys carry the installed user policy.

An unsupported CLI version, added/removed Sol field, wrong value kind, duplicate/missing slug, or preservation mismatch stops plan/install. Each deterministic install manifest records the Codex version, schema ID, raw/normalized/source/preserved/output hashes, approved allowlist, and actual changed fields. See [Headless Linux and Paseo](../LINUX-PASEO.md).
