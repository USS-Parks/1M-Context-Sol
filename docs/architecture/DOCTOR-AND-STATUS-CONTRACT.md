# Doctor and status contract

CAC-12 adds two read-only views of the same deterministic evaluator:

- `cctx doctor` prints every reviewed check plus remediation;
- `cctx status` prints a compact operational summary;
- `--json` emits the schema-validated automation form of either view.

Neither command sends a model request. They inspect the local Codex version, authentication status, exact root model, reviewed configuration fields, runtime feature/catalog diagnostics, operational-threshold state, and strict compaction-guard state. Credential material, arbitrary configuration, model instructions, and raw authentication output are excluded.

## Truth boundary

`policy_ready` means that the inspected configuration policy is internally ready for exact `gpt-5.6-sol`. It does not mean a long request reached the backend. Every CAC-12 report therefore carries these values:

```json
{
  "catalog_is_not_live_proof": true,
  "live_native_window_proven": false,
  "release_claim_ready": false
}
```

A catalog or configuration can report 1,050,000 tokens and a computed effective budget above one million while the G2 live gate remains open. Only the separately authorized, budget-capped live probe can change that evidence state in a later prompt.

## Canonical dimensions

Doctor output uses the six IDs, labels, units, and authorities from the claim contract without substitution:

| ID | Label | CAC-12 observation |
|---|---|---|
| `native_total_context` | Native total context window | Official 1,050,000-token contract beside the resolved catalog policy value; not a live observation |
| `native_max_input` | Native maximum input | Official 922,000-token contract; not separately represented by the catalog |
| `native_max_output` | Native maximum output | Official 128,000-token contract; not separately represented by the catalog |
| `codex_effective_budget` | Effective Codex budget | Checked arithmetic from resolved catalog context and effective percentage |
| `operational_input_threshold` | Operational input threshold | CAC-14 measured value or an explicit pending state |
| `durable_reservoir_capacity` | Durable reservoir capacity | Explicitly not implemented at CAC-12; never added to the native window |

## Deterministic checks

The evaluator emits stable check IDs for capture completeness, Codex compatibility, strict TOML validity, authentication, exact Sol identity, profile overrides, catalog compatibility, catalog Sol identity, one-model/no-fallback shape, catalog total/max policy, configured model window, selected catalog path, effective budget, automatic-compaction policy, operational threshold, strict compaction guard, and separation of policy evidence from live proof.

An exact-Sol policy-ready result requires:

- authenticated ChatGPT or API-key access;
- supported Codex `0.144.5` compatibility profile;
- valid TOML with exact root `gpt-5.6-sol` and zero profile model overrides;
- exactly one resolved catalog model, also exact Sol;
- 1,050,000 context and maximum catalog policy plus explicit 1,050,000 Codex window;
- at least 1,000,000 computed Effective Codex budget;
- a CAC-14 operational threshold below 922,000 and no later than automatic compaction; and
- an observed strict blocking guard whose checkpoint threshold matches the operational threshold.

The current 272,000/95% baseline computes to a 258,400-token Effective Codex budget and correctly returns `not_ready`. A 1.05M/96% overlay computes to 1,008,000 but remains `not_ready` until the operational threshold and blocking guard are proven.

## Exit-code contract

| Code | Meaning |
|---:|---|
| `0` | Configuration policy ready; still not live native-window proof |
| `1` | Runtime inspection or serialization failure |
| `2` | Inspection is trustworthy, but one or more fixable policy checks failed |
| `3` | Unsupported or untrustworthy input, such as invalid TOML, stale catalog profile, or unsupported Codex version |
| `64` | Invalid command usage |

The same code appears in JSON and is returned by the process for codes 0, 2, and 3. Runtime and usage errors are emitted on stderr with codes 1 and 64.

## Schemas and frozen scenarios

- [`schemas/doctor-report.schema.json`](../../schemas/doctor-report.schema.json) covers the detailed report.
- [`schemas/status-report.schema.json`](../../schemas/status-report.schema.json) covers the compact report.
- [`tests/fixtures/doctor-scenarios.json`](../../tests/fixtures/doctor-scenarios.json) freezes eight normalized observations.
- [`tests/golden/doctor-scenarios.json`](../../tests/golden/doctor-scenarios.json) freezes full doctor/status JSON and human-output hashes plus verdicts and blocker IDs.

The golden matrix covers ChatGPT-authenticated Sol at 272k, ChatGPT Sol with the 1.05M replacement catalog, direct-API Sol, a non-Sol override, missing access, a stale catalog, invalid configuration, and an unsupported Codex version. Claim-contract tests reject forbidden wording and require every dimension label and authority to match the canonical vocabulary.

## Commands

```powershell
cctx doctor
cctx doctor --json
cctx status
cctx status --json
```

`--codex <COMMAND>` and `--config <FILE>` select explicit read-only inputs for diagnostics. The commands do not install the catalog, alter global Codex configuration, enter credentials, or exercise a paid model request.

Authoritative references:

- [GPT-5.6 Sol model limits](https://developers.openai.com/api/docs/models/gpt-5.6-sol)
- [Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
- [Codex hooks reference](https://learn.chatgpt.com/docs/hooks)
