# Context Continuum Claim Contract

## Status

This contract is normative for code, tests, documentation, demos, releases, and community posts. It defines what each capacity number means and which gate must pass before a public claim is enabled.

The machine-readable source is `contracts/capability-vocabulary.json`. The current release claim remains disabled: CAC-01 proves the installed runtime is below the native-window goal, and G2–G5 have not yet passed.

## Model identity

The only compliant model slug is exact `gpt-5.6-sol`.

- Model fallback is forbidden.
- Model aliases are not accepted as proof.
- A response, config, catalog, benchmark, or screenshot from any other slug is noncompliant.
- A non-Sol input must cause claim validation to fail before capacity evidence is considered.

## Six separate capacity dimensions

| ID | Public label | Contract | Authority | Gate | May substitute for |
|---|---|---:|---|---|---|
| `native_total_context` | Native total context window | exactly 1,050,000 tokens | OpenAI Sol specification | G2 | Nothing |
| `native_max_input` | Native maximum input | exactly 922,000 tokens | OpenAI Sol specification | G2 | Nothing |
| `native_max_output` | Native maximum output | exactly 128,000 tokens | OpenAI Sol specification | G2 | Nothing |
| `codex_effective_budget` | Effective Codex budget | at least 1,000,000 tokens | resolved client plus live probe | G2 | Nothing |
| `operational_input_threshold` | Operational input threshold | measured, greater than zero, below 922,000 | CAC-14 calibration | G2 | Nothing |
| `durable_reservoir_capacity` | Durable reservoir capacity | at least 1,000,000 tokenizer-counted tokens | G3 benchmark | G3 | Nothing |

The dimensions are intentionally non-interchangeable:

- 1,050,000 total equals 922,000 maximum input plus 128,000 maximum output. It does not mean one million input tokens.
- Effective Codex budget includes client policy and reserve. It does not replace the backend's official limits.
- The operational threshold is lower than maximum input because instructions, tools, and output require headroom.
- Durable capacity is stored outside the active request. It cannot prove or enlarge the native window.

## Claim-to-gate map

No row is enabled before every listed gate passes.

| Claim ID | Permitted wording | Required gates |
|---|---|---|
| `headline` | “GPT-5.6 Sol with its native 1.05M window. Compaction blocked. Durable continuity beyond the window.” | G2, G3, G4, G5 |
| `sol_only` | Every compliant task uses exact `gpt-5.6-sol` with no model fallback. | G2, G5 |
| `native_window` | The active Sol backend exposes its documented 1,050,000-token total context window. | G2 |
| `maximum_input` | Sol has a separate 922,000-token maximum input. | G2 |
| `maximum_output` | Sol has a separate 128,000-token maximum output. | G2 |
| `effective_budget` | Codex resolves at least a 1,000,000-token internal budget while operating below maximum input. | G2 |
| `durable_capacity` | The local reservoir retains at least 1,000,000 tokenizer-counted tokens with integrity and exact retrieval. | G3 |
| `no_compaction` | Known automatic and manual compaction attempts are checkpointed and visibly blocked. | G4 |
| `fresh_task_enrollment` | Every fresh supported Codex task is automatically enrolled or visibly blocked before ordinary work. | G5 |
| `local_private_default` | Ordinary storage and retrieval are local-only and telemetry-off by default. | G6 |
| `release_installable` | The supported release installs from verified GitHub artifacts rather than a developer checkout. | G8, G9 |

Repository planning prose may describe these as targets when it says clearly that the gate remains open. Release-facing prose may not present a target as achieved.

## Limitation table

| Limitation | Required wording consequence |
|---|---|
| Total is not input | Always report 1,050,000 total with 922,000 maximum input and 128,000 maximum output nearby. |
| Catalog is not live proof | A catalog overlay, status screen, or local arithmetic cannot close G2. |
| Reservoir is not native | Never add durable tokens to native tokens or market their sum as a model window. |
| Strict guard can stop | State that blocking `PreCompact` can halt the active turn at the hard limit. |
| Rollover is a new task | Do not describe successor rollover as unlimited same-task liveness. |
| Current runtime is below goal | CAC-01 currently records 272,000 resolved total and 258,400 effective tokens. |
| Long context has premium pricing | Paid probes above 272,000 input require their explicit budget gate. |
| Platform support needs live proof | Compilation alone cannot label a platform supported. |

## Wording rules

The validator rejects wording that claims one million input tokens, unlimited or infinite context, no context limit, or compatibility with any model. It also validates the exact headline, Sol slug, gate map, and numeric relationships from the machine-readable vocabulary.

Examples of compliant target language:

- “1.05M total context, with a separate 922k maximum input.”
- “At least 1M effective Codex budget; calibrated operating threshold below max input.”
- “At least 1M tokens retained in the durable local reservoir.”

Examples that must fail:

- a one-million-input claim;
- an unlimited-context claim;
- adding reservoir capacity to the native model window;
- presenting a custom catalog as live native proof; or
- substituting a different model identity.

## Test contract

`src/claim_contract.rs` and `tests/claim_contract.rs` enforce:

1. all six dimension IDs exist exactly once;
2. official totals and the input/output sum are exact;
3. effective and durable minimums are separate;
4. the operational threshold constraint is below maximum input;
5. every public claim maps to at least one valid G0–G9 gate;
6. the headline maps to G2–G5 and remains disabled before those gates pass;
7. forbidden wording is rejected;
8. a non-Sol model identity is rejected; and
9. checked-in JSON contracts and schemas remain valid JSON.

This contract can be expanded only by a reviewed PSPR addendum. It cannot be weakened after a failed measurement.
