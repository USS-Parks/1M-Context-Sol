# Context Continuum for GPT-5.6 Sol

Context Continuum is an open-source Codex runtime extension in active development for exact model `gpt-5.6-sol`. The planned product combines a Codex plugin, native `cctx` CLI/service, local MCP server, strict compaction guard, and durable local context reservoir.

## Current status

Foundation milestone M0 is complete under the [canonical PSPR](PLANNING/CODEX-CONTEXT-CONTINUUM-PSPR.md), and M1 implementation is underway. No native one-million-window claim has passed its live gate yet, and this repository does not currently provide an installable release.

The frozen [CAC-01 capability baseline](docs/architecture/CODEX-CAPABILITY-BASELINE.md) shows `codex-cli 0.144.5` bundling Sol at 372,000 tokens but resolving Sol at 272,000 with a 258,400-token effective budget. That reproduces the current “256k class” behavior and remains far below Sol's official 1,050,000-token total window.

The release contract, once proven, is:

> GPT-5.6 Sol with its native 1.05M window. Compaction blocked. Durable continuity beyond the window.

That sentence is a gated target, not a current performance claim. The project always reports Sol's 1,050,000-token total window separately from its 922,000-token maximum input and 128,000-token maximum output.

## Governance

Implementation follows stable CAC prompt IDs, prompt-local acceptance gates, and an auditable [development log](docs/sessions/CODEX-CONTEXT-CONTINUUM-DEVLOG.md). Public wording and capacity numbers are governed by the [claim contract](docs/architecture/CLAIM-CONTRACT.md) and its [machine-readable vocabulary](contracts/capability-vocabulary.json). See [verification status](docs/VERIFICATION.md) before relying on any capability.

The settled component ownership and trust transitions are documented in the [data-flow contract](docs/architecture/DATA-FLOW-AND-TRUST-BOUNDARIES.md), nine [architecture decisions](docs/architecture/decisions/), and the repository [threat model](docs/security/THREAT-MODEL.md).

The [CI and evidence contract](docs/CI-AND-EVIDENCE-CONTRACT.md) requires clean Rust, documentation, secret, supply-chain, PSPR, and Draft 2020-12 JSON Schema gates plus deliberate failure proofs.

The version-pinned [Sol catalog overlay](docs/architecture/MODEL-CATALOG-OVERLAY.md) now generates an uninstalled one-model candidate while preserving installed instructions and capabilities. That local catalog/parser proof is not live backend acceptance.

The [atomic configuration manager](docs/architecture/ATOMIC-CONFIG-MANAGEMENT.md) can now dry-run, atomically apply, and exactly roll back only its declared model, window, catalog, hook/plugin, and MCP settings. It requires explicit absolute paths and has not been run against the real global Codex configuration.

The claim-safe [`doctor` and `status`](docs/architecture/DOCTOR-AND-STATUS-CONTRACT.md) commands now expose exact Sol identity, authentication lane, official limits, resolved catalog policy, Effective Codex budget, operational threshold, and compaction-guard state in human and JSON forms. They deliberately distinguish configuration-policy readiness from still-open live native-window proof.

The [strict startup policy](docs/architecture/STRICT-SOL-STARTUP-POLICY.md) now implements fail-closed `SessionStart` and `UserPromptSubmit` handling. It blocks any non-Sol active model, model override, non-green doctor state, missing access, 272k/stale catalog, missing calibrated threshold, or unproven compaction guard, and records a prompt-free per-task audit. It is implemented and tested but is not globally installed yet.

## License

Apache-2.0. See [LICENSE](LICENSE).
